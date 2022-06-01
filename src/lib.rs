use quick_js::{Context, JsValue};
use swc_plugin::{ast::*, plugin_transform, TransformPluginProgramMetadata, chain, syntax_pos::Mark};

use swc_ecma_parser::Syntax;
use swc_ecma_transforms_testing::test;
use swc_ecma_transforms_base::resolver;

pub struct TransformVisitor;

impl VisitMut for TransformVisitor {
    fn visit_mut_expr(&mut self, n: &mut Expr) {
        n.visit_mut_children_with(self);

        if let Expr::TaggedTpl(tagged_tpl) = &*n {
            if let Expr::Ident(ident) = &*tagged_tpl.tag {
                if &*ident.sym == "eval" {
                    let context = Context::new().unwrap();
                    let js_expression: String = (&tagged_tpl.tpl.quasis[0].raw).to_string();
                    let jsvalue = context.eval(&js_expression).unwrap();

                    let new_expr = match jsvalue {
                        JsValue::String(v) => Expr::Lit(Lit::Str(Str {
                            span: ident.span,
                            value: v.into(),
                            raw: None,
                        })),
                        JsValue::Int(v) => Expr::Lit(Lit::Num(Number {
                            span: ident.span,
                            value: v.into(),
                            raw: None,
                        })),
                        _ => unreachable!() // Not implemented
                    };

                    *n = new_expr;
                }
            }
        }
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor))
}

fn tr() -> impl Fold {
    chain!(
        resolver(Mark::new(), Mark::new(), false),
        as_folder(TransformVisitor)
    )
}

test!(
    Syntax::Es(Default::default()),
    |_| tr(),
    string,
    "let three = eval`(1+2).toString()`;",
    "let three = \"3\";"
);

test!(
    Syntax::Es(Default::default()),
    |_| tr(),
    number,
    "let three = eval`1+2`;",
    "let three = 3;"
);
