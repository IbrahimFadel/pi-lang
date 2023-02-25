use std::{
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Index,
};

use flux_span::FileId;
use flux_syntax::ast::{self, AstNode, Root};
use la_arena::{Arena, Idx};
use lasso::ThreadedRodeo;

use crate::{
    hir::{Apply, Enum, Function, Mod, Struct, Trait},
    ModuleDefId, TypeInterner,
};

mod lower;

#[derive(Debug, Default)]
pub struct ItemTree {
    pub top_level: Vec<ModItem>,
    applies: Arena<Apply>,
    enums: Arena<Enum>,
    functions: Arena<Function>,
    mods: Arena<Mod>,
    structs: Arena<Struct>,
    traits: Arena<Trait>,
}

impl ItemTree {}

pub trait ItemTreeNode: Clone {
    type Source: AstNode + Into<ast::Item>;

    /// Looks up an instance of `Self` in an item tree.
    fn lookup(tree: &ItemTree, index: Idx<Self>) -> &Self;

    /// Downcasts a `ModItem` to a `FileItemTreeId` specific to this type.
    fn id_from_mod_item(mod_item: ModItem) -> Option<FileItemTreeId<Self>>;

    /// Upcasts a `FileItemTreeId` to a generic `ModItem`.
    fn id_to_mod_item(id: FileItemTreeId<Self>) -> ModItem;
}

pub struct FileItemTreeId<N: ItemTreeNode> {
    index: Idx<N>,
    _p: PhantomData<N>,
}

impl From<FileItemTreeId<Function>> for ModuleDefId {
    fn from(value: FileItemTreeId<Function>) -> Self {
        ModuleDefId::FunctionId(value.index)
    }
}

impl<N: ItemTreeNode> Clone for FileItemTreeId<N> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            _p: PhantomData,
        }
    }
}
impl<N: ItemTreeNode> Copy for FileItemTreeId<N> {}

impl<N: ItemTreeNode> PartialEq for FileItemTreeId<N> {
    fn eq(&self, other: &FileItemTreeId<N>) -> bool {
        self.index == other.index
    }
}
impl<N: ItemTreeNode> Eq for FileItemTreeId<N> {}

impl<N: ItemTreeNode> Hash for FileItemTreeId<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

impl<N: ItemTreeNode> fmt::Debug for FileItemTreeId<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.index.fmt(f)
    }
}

impl<N: ItemTreeNode> Index<FileItemTreeId<N>> for ItemTree {
    type Output = N;
    fn index(&self, id: FileItemTreeId<N>) -> &N {
        N::lookup(self, id.index)
    }
}

macro_rules! mod_items {
    ( $( $typ:ident in $fld:ident -> $ast:ty ),+ $(,)? ) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum ModItem {
            $(
                $typ(FileItemTreeId<$typ>),
            )+
        }

        $(
            impl From<FileItemTreeId<$typ>> for ModItem {
                fn from(id: FileItemTreeId<$typ>) -> ModItem {
                    ModItem::$typ(id)
                }
            }
        )+

        $(
            impl ItemTreeNode for $typ {
                type Source = $ast;

                fn lookup(tree: &ItemTree, index: Idx<Self>) -> &Self {
                    &tree.$fld[index]
                }

                fn id_from_mod_item(mod_item: ModItem) -> Option<FileItemTreeId<Self>> {
                    match mod_item {
                        ModItem::$typ(id) => Some(id),
                        _ => None,
                    }
                }

                fn id_to_mod_item(id: FileItemTreeId<Self>) -> ModItem {
                    ModItem::$typ(id)
                }
            }

            impl Index<Idx<$typ>> for ItemTree {
                type Output = $typ;

                fn index(&self, index: Idx<$typ>) -> &Self::Output {
                    &self.$fld[index]
                }
            }
        )+
    };
}

mod_items! {
    Apply in applies -> ast::ApplyDecl,
    Enum in enums -> ast::EnumDecl,
    Function in functions -> ast::FnDecl,
    Mod in mods -> ast::ModDecl,
    Struct in structs -> ast::StructDecl,
    Trait in traits -> ast::TraitDecl,
}

pub fn lower_ast_to_item_tree(
    root: &Root,
    file_id: FileId,
    string_interner: &'static ThreadedRodeo,
    type_interner: &'static TypeInterner,
) -> ItemTree {
    lower::Ctx::new(file_id, string_interner, type_interner).lower_module_items(root)
}