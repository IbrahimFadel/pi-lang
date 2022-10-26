use super::*;
use crate::getters;

// stupidly unnecessary macro syntax, but... pretty? prettier i guess
getters! {
    Root {
        fn_decls -> nodes(FnDecl);
        struct_decls -> nodes(StructDecl);
        enum_decls -> nodes(EnumDecl);
        trait_decls -> nodes(TraitDecl);
        apply_decls -> nodes(ApplyDecl);
    }
    Visibility {
        public -> tok(Pub);
    }
    FnDecl {
        fn_kw -> tok(Fn);
        name -> node(Name);
        generic_param_list -> node(GenericParamList);
        where_clause -> node(WhereClause);
        visibility -> node(Visibility);
        param_list -> node(ParamList);
        return_type -> node(Type);
        body -> node(Expr);
    }
    Name {
        ident -> tok(Ident);
    }
    ParamList {
        params -> nodes(Param);
    }
    Param {
        ty -> node(Type);
        name -> tok(Ident);
    }
    PathType {
        path -> node(Path);
    }
    TupleType {
        types -> nodes(Type);
    }
    ArrayType {
        ty -> node(Type);
        n -> node(IntExpr);
    }
    PtrType {
        ty -> node(Type);
    }
    Path {
        segments -> toks(Ident);
    }

    IntExpr {
        v -> tok(IntLit);
    }
    FloatExpr {
        v -> tok(FloatLit);
    }
    PathExpr {
        segments -> toks(Ident);
    }
    BlockExpr {
        stmts -> nodes(Stmt);
    }
    CallExpr {
        path -> node(PathExpr);
        args -> node(ArgList);
        lparen -> tok(LParen);
        rparen -> tok(RParen);
    }
    ArgList {
        args -> nodes(Expr);
    }

    LetStmt {
        name -> node(Name);
        ty -> node(Type);
        value -> node(Expr);
    }
    ExprStmt {
        expr -> node(Expr);
        semicolon -> tok(SemiColon);
    }
    StructDecl {
        name -> node(Name);
        generic_param_list -> node(GenericParamList);
        where_clause -> node(WhereClause);
        field_list -> node(StructDeclFieldList);
    }
    StructDeclFieldList {
        fields -> nodes(StructDeclField);
    }
    StructDeclField {
        name -> node(Name);
        ty -> node(Type);
    }
    GenericParamList {
        type_params -> nodes(TypeParam);
    }
    TypeParam {
        name -> tok(Ident);
    }
    WhereClause {
        predicates -> nodes(WherePredicate);
    }
    WherePredicate {
        name -> tok(Ident);
        type_bound_list -> node(TypeBoundList);
    }
    TypeBoundList {
        type_bounds -> nodes(TypeBound);
    }
    TypeBound {
        trait_name -> tok(Ident);
        generic_arg_list -> node(GenericArgList);
    }
    GenericArgList {
        args -> nodes(Type);
    }
}
