use crate::model::{Index, Symbol, SymbolKind, Visibility};
use std::cmp::Ordering;
use std::fmt::Write;

#[derive(Debug, Clone, Copy, Default)]
pub struct RenderOptions {
    pub include_private: bool,
}

#[derive(Debug, Default)]
pub struct SkeletonRenderer;

impl SkeletonRenderer {
    pub fn render(&self, index: &Index, options: RenderOptions) -> String {
        let mut output = String::new();
        let mut files = index.files.clone();
        files.sort_by(|a, b| a.path.cmp(&b.path));

        for file in files {
            let mut symbols: Vec<Symbol> = file
                .symbols
                .into_iter()
                .filter(|symbol| should_include(symbol, options))
                .collect();
            if symbols.is_empty() {
                continue;
            }

            symbols.sort_by(symbol_sort);
            let _ = writeln!(&mut output, "# {}", file.path.display());
            for symbol in symbols {
                let _ = writeln!(&mut output, "- {}", symbol.signature);
            }
            let _ = writeln!(&mut output);
        }

        output
    }
}

fn should_include(symbol: &Symbol, options: RenderOptions) -> bool {
    if options.include_private {
        return true;
    }

    !matches!(symbol.visibility, Visibility::Private)
}

fn symbol_sort(a: &Symbol, b: &Symbol) -> Ordering {
    let a_rank = kind_rank(a.kind);
    let b_rank = kind_rank(b.kind);
    a_rank.cmp(&b_rank).then_with(|| a.name.cmp(&b.name))
}

fn kind_rank(kind: SymbolKind) -> u8 {
    match kind {
        SymbolKind::Module => 0,
        SymbolKind::Struct => 1,
        SymbolKind::Enum => 2,
        SymbolKind::Trait => 3,
        SymbolKind::TypeAlias => 4,
        SymbolKind::Const => 5,
        SymbolKind::Function => 6,
        SymbolKind::Method => 7,
    }
}
