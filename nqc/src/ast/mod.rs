// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::lexer::Tokens;
//     use insta::{assert_debug_snapshot, assert_snapshot, glob};

//     #[test]
//     fn snapshot_tests() {
//         glob!("../../tests/good", "*.nqc", |path| {
//             let src = std::fs::read_to_string(path).unwrap();
//             let tokens = Tokens::new(&src).unwrap();
//             let stream = tokens.iter();
//             let ast = Ast::parse(stream).map_err(miette::Report::from).unwrap();
//             assert_debug_snapshot!(ast);
//         });
//     }

//     #[test]
//     fn error_snapshot_tests() {
//         std::env::set_var("NO_COLOR", "true");
//         glob!("../../tests/bad", "*.nqc", |path| {
//             let src = std::fs::read_to_string(path).unwrap();
//             let tokens = Tokens::new(&src).unwrap();
//             let stream = tokens.iter();
//             let err = miette::Report::from(Ast::parse(stream).unwrap_err());
//             assert_snapshot!(format!("{err:?}"));
//         });
//     }
// }
