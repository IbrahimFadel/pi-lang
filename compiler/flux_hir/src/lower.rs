use std::collections::HashMap;

use flux_span::{Span, Spanned, ToSpan, WithSpan};
use flux_syntax::{
    ast::{self, AstNode, Root},
    SyntaxNode,
};
use itertools::Itertools;
use la_arena::Arena;
use lasso::{Spur, ThreadedRodeo};

use crate::{
    hir::{
        Block, Expr, ExprIdx, FnDecl, FunctionId, GenericParamList, ItemDefinitionId, Let, ModDecl,
        Module, ModuleId, Name, Param, ParamList, Path, Struct, StructDecl, StructField,
        StructFieldList, Type, TypeBound, TypeBoundList, UseDecl, Visibility, WhereClause,
        WherePredicate, WithType,
    },
    type_interner::{TypeIdx, TypeInterner},
};

use self::bodies::ModuleBodyContext;

const POISONED_NAME: &str = "POISONED";

mod bodies;

struct Context<'a> {
    module_path: Vec<Spur>,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
    exprs: Arena<Spanned<Expr>>,
    function_namespace: &'a mut HashMap<Spur, (FunctionId, ModuleId)>,
}

impl<'a> Context<'a> {
    pub fn new(
        module_path: Vec<Spur>,
        string_interner: &'static ThreadedRodeo,
        type_interner: &'static TypeInterner,
        function_namespace: &'a mut HashMap<Spur, (FunctionId, ModuleId)>,
    ) -> Self {
        Self {
            module_path,
            string_interner,
            type_interner,
            exprs: Arena::new(),
            function_namespace,
        }
    }

    /// Lower an AST node to its HIR equivalent
    ///
    /// This exists to help clean up the lowering process due to the optional nature of the AST layer.
    /// We want certain nodes to **ALWAYS** be emitted even when there's a parsing error, but be marked as poisoned.
    /// For this reason, we can `unwrap`/`expect` safely (panics are ICEs), then carry on.
    ///
    /// If the node is poisoned, use the supplied closure to provide a poisoned value.
    /// If the node is not poisoned, use the supplied closure to carry out the regular lowering process.
    ///
    /// This method can be quite verbose and clog up code, so generally this should be used in generalizable methods such as `lower_name` or `lower_generic_param_list`, not in unique methods such as `lower_fn_decl`.
    // fn lower_node<N, T, P, F>(
    //     &mut self,
    //     node: Option<N>,
    //     poison_function: P,
    //     normal_function: F,
    // ) -> T
    // where
    //     N: AstNode,
    //     P: FnOnce(&mut Self, N) -> T,
    //     F: FnOnce(&mut Self, N) -> T,
    // {
    //     let n = node.expect("internal compiler error: missing node that should always be emitted");
    //     if n.is_poisoned() {
    //         poison_function(self, n)
    //     } else {
    //         normal_function(self, n)
    //     }
    // }

    // fn lower_name(&mut self, name: Option<ast::Name>) -> Name {
    //     self.lower_node(
    //         name,
    //         |_, name| {
    //             self.string_interner
    //                 .get_or_intern_static(POISONED_NAME)
    //                 .at(name.range().to_span())
    //         },
    //         |_, name| {
    //             name.ident().expect("internal compiler error: name did not contain identifier but was not marked poisoned").text_key().at(name.range().to_span())
    //         },
    //     )
    // }

    // fn lower_path(&mut self, path: Option<ast::Path>) -> Spanned<Path> {
    //     self.lower_node(
    //         path,
    //         |this, path| Path::poisoned().at(path.range().to_span()),
    //         |this, path| {
    //             Path::from_segments(
    //                 path.segments()
    //                     .map(|tok| tok.text_key().at(tok.text_range().to_span())),
    //             )
    //             .at(path.range().to_span())
    //         },
    //     )
    // }

    // fn lower_visibility(&mut self, visibility: Option<ast::Visibility>) -> Visibility {
    //     self.lower_node(
    //         visibility,
    //         |_, _| Visibility::Private,
    //         |_, visibility| {
    //             visibility
    //                 .public()
    //                 .map_or(Visibility::Private, |_| Visibility::Public)
    //         },
    //     )
    // }

    // fn lower_generic_param_list(
    //     &mut self,
    //     generic_param_list: Option<ast::GenericParamList>,
    // ) -> GenericParamList {
    //     if let Some(generic_param_list) = generic_param_list {
    //         let type_params = generic_param_list
    //             .type_params()
    //             .map(|type_param| self.lower_name(type_param.name()))
    //             .collect();
    //         GenericParamList::new(type_params)
    //     } else {
    //         GenericParamList::empty()
    //     }
    // }

    fn lower_where_clause(
        &mut self,
        where_clause: Option<ast::WhereClause>,
        generic_param_list: &GenericParamList,
    ) -> WhereClause {
        if let Some(where_clause) = where_clause {
            let predicates = where_clause
                .predicates()
                .map(|where_predicate| {
                    self.lower_where_predicate(where_predicate, generic_param_list)
                })
                .collect();
            WhereClause::new(predicates)
        } else {
            WhereClause::EMPTY
        }
    }

    fn lower_where_predicate(
        &mut self,
        where_predicate: ast::WherePredicate,
        generic_param_list: &GenericParamList,
    ) -> WherePredicate {
        let generic = lower_name(where_predicate.name(), self.string_interner);
        let trait_restrictions = lower_node(
            where_predicate.type_bound_list(),
            |_| TypeBoundList::EMPTY,
            |type_bound_list| self.lower_type_bound_list(type_bound_list, generic_param_list),
        );
        WherePredicate::with_trait_restrictions(generic, trait_restrictions)
    }

    fn lower_type_bound_list(
        &mut self,
        type_bound_list: ast::TypeBoundList,
        generic_param_list: &GenericParamList,
    ) -> TypeBoundList {
        TypeBoundList::new(
            type_bound_list
                .type_bounds()
                .map(|type_bound| self.lower_type_bound(type_bound, generic_param_list))
                .collect(),
        )
    }

    fn lower_type_bound(
        &mut self,
        type_bound: ast::TypeBound,
        generic_params_list: &GenericParamList,
    ) -> TypeBound {
        let name = lower_name(type_bound.trait_name(), self.string_interner);
        let generic_arg_list = lower_generic_arg_list(
            type_bound.generic_arg_list(),
            generic_params_list,
            self.string_interner,
            self.type_interner,
        );
        TypeBound::with_args(name, generic_arg_list)
    }

    // fn lower_generic_arg_list(
    //     &mut self,
    //     generic_arg_list: Option<ast::GenericArgList>,
    //     generic_params_list: &GenericParamList,
    // ) -> Vec<Spanned<TypeIdx>> {
    //     if let Some(generic_arg_list) = generic_arg_list {
    //         generic_arg_list
    //             .args()
    //             .map(|ty| {
    //                 let span = ty.range().to_span();
    //                 self.lower_type(Some(ty), generic_params_list, span)
    //             })
    //             .collect()
    //     } else {
    //         vec![]
    //     }
    // }

    // fn lower_type(
    //     &mut self,
    //     ty: Option<ast::Type>,
    //     generic_param_list: &GenericParamList,
    //     fallback_span: Span,
    // ) -> Spanned<TypeIdx> {
    //     self.lower_node(
    //         ty,
    //         |this, _| {
    //             this.type_interner
    //                 .intern(Type::Tuple(vec![]))
    //                 .at(fallback_span)
    //         },
    //         |this, ty| match ty {
    //             ast::Type::ArrayType(arr) => this.lower_array_type(arr, generic_param_list),
    //             ast::Type::PathType(path_type) => {
    //                 this.lower_path_or_generic_type(path_type, generic_param_list)
    //             }
    //             ast::Type::PtrType(ptr) => this.lower_ptr_type(ptr, generic_param_list),
    //             ast::Type::TupleType(tuple) => this.lower_tuple_type(tuple, generic_param_list),
    //         },
    //     )
    // }

    // fn lower_array_type(
    //     &mut self,
    //     arr: ast::ArrayType,
    //     generic_param_list: &GenericParamList,
    // ) -> Spanned<TypeIdx> {
    //     let span = arr.range().to_span();
    //     let ty = self.lower_type(arr.ty(), generic_param_list, span);
    //     let n: Spanned<u32> = self.lower_node(
    //         arr.n(),
    //         |_, _| 0.at(span),
    //         |_, int| {
    //             let span = int.range().to_span();
    //             let value_str = match int.v() {
    //                 Some(v) => self
    //                     .string_interner
    //                     .resolve(&v.text_key())
    //                     .at(v.text_range().to_span()),
    //                 None => "0".at(span),
    //             };
    //             value_str.map(|v| match v.parse() {
    //                 Ok(v) => v,
    //                 Err(_) => todo!(),
    //             })
    //         },
    //     );
    //     let ty = Type::Array(ty, n);
    //     self.type_interner.intern(ty).at(span)
    // }

    // fn lower_path_or_generic_type(
    //     &mut self,
    //     path_ty: ast::PathType,
    //     generic_param_list: &GenericParamList,
    // ) -> Spanned<TypeIdx> {
    //     let path = self.lower_path(path_ty.path());
    //     let args = self.lower_generic_arg_list(path_ty.generic_arg_list(), generic_param_list);
    //     let ty = if path.len() > 1 {
    //         Type::Path(path.inner, args)
    //     } else if generic_param_list
    //         .get(path.nth(0).expect("internal compiler error: path is empty"))
    //     {
    //         Type::Generic(
    //             *path
    //                 .get_unspanned_spurs()
    //                 .collect::<Vec<_>>()
    //                 .first()
    //                 .unwrap(),
    //         )
    //     } else {
    //         Type::Path(path.inner, args)
    //     };

    //     self.type_interner.intern(ty).at(path_ty.range().to_span())
    // }

    // fn lower_ptr_type(
    //     &mut self,
    //     ptr: ast::PtrType,
    //     generic_param_list: &GenericParamList,
    // ) -> Spanned<TypeIdx> {
    //     let span = ptr.range().to_span();
    //     let ty = self.lower_type(ptr.ty(), generic_param_list, span);
    //     let ty = Type::Ptr(ty.at(span));
    //     self.type_interner.intern(ty).at(span)
    // }

    // fn lower_tuple_type(
    //     &mut self,
    //     tuple: ast::TupleType,
    //     generic_param_list: &GenericParamList,
    // ) -> Spanned<TypeIdx> {
    //     let types: Vec<_> = tuple
    //         .types()
    //         .map(|ty| {
    //             let span = ty.range().to_span();
    //             self.lower_type(Some(ty), generic_param_list, span)
    //         })
    //         .collect();
    //     let ty = Type::Tuple(types);
    //     self.type_interner.intern(ty).at(tuple.range().to_span())
    // }

    fn lower_struct_field_list(
        &mut self,
        field_list: Option<ast::StructDeclFieldList>,
        generic_param_list: &GenericParamList,
    ) -> StructFieldList {
        lower_node(
            field_list,
            |_| StructFieldList::empty(),
            |field_list| {
                let fields = field_list
                    .fields()
                    .map(|field| self.lower_struct_field(field, generic_param_list))
                    .collect();
                StructFieldList::new(fields)
            },
        )
    }

    fn lower_struct_field(
        &mut self,
        field: ast::StructDeclField,
        generic_param_list: &GenericParamList,
    ) -> StructField {
        let ty = lower_type(
            field.ty(),
            generic_param_list,
            field.range().to_span(),
            self.string_interner,
            self.type_interner,
        );
        let name = lower_name(field.name(), self.string_interner);
        StructField::new(name, ty)
    }

    fn lower_fn_param_list(
        &mut self,
        param_list: Option<ast::ParamList>,
        generic_param_list: &GenericParamList,
    ) -> ParamList {
        lower_node(
            param_list,
            |_| ParamList::empty(),
            |param_list| {
                let params = param_list
                    .params()
                    .map(|param| self.lower_fn_param(param, generic_param_list))
                    .collect();
                ParamList::new(params)
            },
        )
    }

    fn lower_fn_param(
        &mut self,
        param: ast::Param,
        generic_param_list: &GenericParamList,
    ) -> Param {
        let name = lower_name(param.name(), self.string_interner);
        let ty = lower_type(
            param.ty(),
            generic_param_list,
            param.range().to_span(),
            self.string_interner,
            self.type_interner,
        );
        Param::new(name, ty)
    }

    fn lower_fn_return_type(
        &mut self,
        ty: Option<ast::Type>,
        generic_param_list: &GenericParamList,
        fallback_span: Span,
    ) -> Spanned<TypeIdx> {
        if let Some(ty) = ty {
            lower_type(
                Some(ty),
                generic_param_list,
                fallback_span,
                self.string_interner,
                self.type_interner,
            )
        } else {
            self.type_interner
                .intern(Type::Tuple(vec![]))
                .at(fallback_span)
        }
    }

    fn lower_mod_decl(&mut self, mod_decl: ast::ModDecl) -> ModDecl {
        let name = lower_name(mod_decl.name(), self.string_interner);
        ModDecl::new(name)
    }

    fn lower_use_decl(&mut self, use_decl: ast::UseDecl) -> UseDecl {
        let path = lower_path(use_decl.path());
        UseDecl::new(path.inner)
    }

    fn lower_struct_decl(&mut self, struct_decl: ast::StructDecl) -> StructDecl {
        let name = lower_name(struct_decl.name(), self.string_interner);
        let visibility = lower_visibility(struct_decl.visibility());
        let generic_param_list =
            lower_generic_param_list(struct_decl.generic_param_list(), self.string_interner);
        let where_clause = self.lower_where_clause(struct_decl.where_clause(), &generic_param_list);
        let field_list =
            self.lower_struct_field_list(struct_decl.field_list(), &generic_param_list);
        StructDecl::new(
            visibility,
            name,
            generic_param_list,
            where_clause,
            field_list,
        )
    }

    fn lower_fn_decl(&mut self, fn_decl: ast::FnDecl) -> FnDecl {
        let name = lower_name(fn_decl.name(), self.string_interner);
        let visibility = lower_visibility(fn_decl.visibility());
        let generic_param_list =
            lower_generic_param_list(fn_decl.generic_param_list(), self.string_interner);
        let where_clause = self.lower_where_clause(fn_decl.where_clause(), &generic_param_list);
        let params = self.lower_fn_param_list(fn_decl.param_list(), &generic_param_list);
        let ret_type = self.lower_fn_return_type(
            fn_decl.return_type(),
            &generic_param_list,
            fn_decl.range().to_span(),
        );
        // let body = self.lower_expr(fn_decl.body()).ty_unknown();
        FnDecl::new(
            visibility,
            name,
            generic_param_list,
            params,
            ret_type,
            where_clause,
            fn_decl, // body,
        )
    }

    fn lower_item_declarations(mut self, root: Root, module_id: ModuleId) -> Module {
        let mut module = Module::default();
        root.mod_decls().for_each(|m| {
            module.mods.alloc(self.lower_mod_decl(m));
        });
        root.fn_decls().for_each(|f| {
            let f = self.lower_fn_decl(f);
            self.module_path.push(f.name.inner);
            let idx = module.functions.alloc(f);
            self.function_namespace.insert(
                join_spurs(&self.module_path, self.string_interner),
                (idx, module_id),
            );
            self.module_path.pop();
        });
        module.exprs = self.exprs;
        module
    }
}

fn join_spurs(spurs: &[Spur], interner: &'static ThreadedRodeo) -> Spur {
    interner.get_or_intern(spurs.iter().map(|spur| interner.resolve(spur)).join("::"))
}

fn lower_node<N, T, P, F>(node: Option<N>, poison_function: P, normal_function: F) -> T
where
    N: AstNode,
    P: FnOnce(N) -> T,
    F: FnOnce(N) -> T,
{
    let n = node.expect("internal compiler error: missing node that should always be emitted");
    if n.is_poisoned() {
        poison_function(n)
    } else {
        normal_function(n)
    }
}

fn lower_name(name: Option<ast::Name>, string_interner: &'static ThreadedRodeo) -> Name {
    lower_node(
        name,
        |name| {
            string_interner
                .get_or_intern_static(POISONED_NAME)
                .at(name.range().to_span())
        },
        |name| {
            name.ident().expect("internal compiler error: name did not contain identifier but was not marked poisoned").text_key().at(name.range().to_span())
        },
    )
}

fn lower_path(path: Option<ast::Path>) -> Spanned<Path> {
    lower_node(
        path,
        |path| Path::poisoned().at(path.range().to_span()),
        |path| {
            Path::from_segments(
                path.segments()
                    .map(|tok| tok.text_key().at(tok.text_range().to_span())),
            )
            .at(path.range().to_span())
        },
    )
}

fn lower_type(
    ty: Option<ast::Type>,
    generic_param_list: &GenericParamList,
    fallback_span: Span,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
) -> Spanned<TypeIdx> {
    lower_node(
        ty,
        |_| type_interner.intern(Type::Tuple(vec![])).at(fallback_span),
        |ty| match ty {
            ast::Type::ArrayType(arr) => {
                lower_array_type(arr, generic_param_list, string_interner, type_interner)
            }
            ast::Type::PathType(path_type) => lower_path_or_generic_type(
                path_type,
                generic_param_list,
                string_interner,
                type_interner,
            ),
            ast::Type::PtrType(ptr) => {
                lower_ptr_type(ptr, generic_param_list, string_interner, type_interner)
            }
            ast::Type::TupleType(tuple) => {
                lower_tuple_type(tuple, generic_param_list, string_interner, type_interner)
            }
        },
    )
}

fn lower_array_type(
    arr: ast::ArrayType,
    generic_param_list: &GenericParamList,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
) -> Spanned<TypeIdx> {
    let span = arr.range().to_span();
    let ty = lower_type(
        arr.ty(),
        generic_param_list,
        span,
        string_interner,
        type_interner,
    );
    let n: Spanned<u32> = lower_node(
        arr.n(),
        |_| 0.at(span),
        |int| {
            let span = int.range().to_span();
            let value_str = match int.v() {
                Some(v) => string_interner
                    .resolve(&v.text_key())
                    .at(v.text_range().to_span()),
                None => "0".at(span),
            };
            value_str.map(|v| match v.parse() {
                Ok(v) => v,
                Err(_) => todo!(),
            })
        },
    );
    let ty = Type::Array(ty, n);
    type_interner.intern(ty).at(span)
}

fn lower_path_or_generic_type(
    path_ty: ast::PathType,
    generic_param_list: &GenericParamList,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
) -> Spanned<TypeIdx> {
    let path = lower_path(path_ty.path());
    let args = lower_generic_arg_list(
        path_ty.generic_arg_list(),
        generic_param_list,
        string_interner,
        type_interner,
    );
    let ty = if path.len() > 1 {
        Type::Path(path.inner, args)
    } else if generic_param_list.get(path.nth(0).expect("internal compiler error: path is empty")) {
        Type::Generic(
            *path
                .get_unspanned_spurs()
                .collect::<Vec<_>>()
                .first()
                .unwrap(),
        )
    } else {
        Type::Path(path.inner, args)
    };

    type_interner.intern(ty).at(path_ty.range().to_span())
}

fn lower_generic_arg_list(
    generic_arg_list: Option<ast::GenericArgList>,
    generic_params_list: &GenericParamList,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
) -> Vec<Spanned<TypeIdx>> {
    if let Some(generic_arg_list) = generic_arg_list {
        generic_arg_list
            .args()
            .map(|ty| {
                let span = ty.range().to_span();
                lower_type(
                    Some(ty),
                    generic_params_list,
                    span,
                    string_interner,
                    type_interner,
                )
            })
            .collect()
    } else {
        vec![]
    }
}

fn lower_ptr_type(
    ptr: ast::PtrType,
    generic_param_list: &GenericParamList,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
) -> Spanned<TypeIdx> {
    let span = ptr.range().to_span();
    let ty = lower_type(
        ptr.ty(),
        generic_param_list,
        span,
        string_interner,
        type_interner,
    );
    let ty = Type::Ptr(ty.at(span));
    type_interner.intern(ty).at(span)
}

fn lower_tuple_type(
    tuple: ast::TupleType,
    generic_param_list: &GenericParamList,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
) -> Spanned<TypeIdx> {
    let types: Vec<_> = tuple
        .types()
        .map(|ty| {
            let span = ty.range().to_span();
            lower_type(
                Some(ty),
                generic_param_list,
                span,
                string_interner,
                type_interner,
            )
        })
        .collect();
    let ty = Type::Tuple(types);
    type_interner.intern(ty).at(tuple.range().to_span())
}

fn lower_visibility(visibility: Option<ast::Visibility>) -> Visibility {
    lower_node(
        visibility,
        |_| Visibility::Private,
        |visibility| {
            visibility
                .public()
                .map_or(Visibility::Private, |_| Visibility::Public)
        },
    )
}

fn lower_generic_param_list(
    generic_param_list: Option<ast::GenericParamList>,
    string_interner: &'static ThreadedRodeo,
) -> GenericParamList {
    if let Some(generic_param_list) = generic_param_list {
        let type_params = generic_param_list
            .type_params()
            .map(|type_param| lower_name(type_param.name(), string_interner))
            .collect();
        GenericParamList::new(type_params)
    } else {
        GenericParamList::empty()
    }
}

pub fn lower_ast_to_hir(
    root: SyntaxNode,
    module_path: Vec<Spur>,
    module_id: ModuleId,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
    mod_namespace: &mut HashMap<Spur, ModuleId>,
    function_namespace: &mut HashMap<Spur, (FunctionId, ModuleId)>,
) -> Module {
    let root =
        ast::Root::cast(root).expect("internal compiler error: root node should always cast");
    mod_namespace.insert(join_spurs(&module_path, string_interner), module_id);
    let ctx = Context::new(
        module_path,
        string_interner,
        type_interner,
        function_namespace,
    );
    ctx.lower_item_declarations(root, module_id)
}

pub fn lower_hir_item_bodies(
    module: &mut Module,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
    mod_namespace: &HashMap<Spur, ModuleId>,
    function_namespace: &HashMap<Spur, (FunctionId, ModuleId)>,
) {
    let mut ctx = ModuleBodyContext::new(
        module,
        string_interner,
        type_interner,
        mod_namespace,
        function_namespace,
    );
    ctx.lower_bodies();
}
