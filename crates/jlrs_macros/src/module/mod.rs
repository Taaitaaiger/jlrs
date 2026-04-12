//! `julia_module!` macro implementation
//!
//! The macro content is converted to Rust code in several steps.
//!
//! 1. Parsing
//!
//! The macro content is first parsed to `JuliaModuleAst`, every node in this AST is identified
//! by a keyword for easy parsing. Next, this raw AST is converted to `ExpandedJuliaModule`; all
//! `ForAst` and `PubAst` nodes are converted to properties of the expanded AST nodes at this
//! point.
//!
//! 2. Modeling
//!
//! The AST is converted to `JuliaModuleModel` which allows for easier property checking. During this phase,
//! items are grouped by export name. Items exported with an environment are expanded further to
//! get rid of all type parameters.
//!
//! 3. Lowering
//!
//! The model is lowered to `JuliaModuleIR` which contains all data that is necessary to
//! effectively generate code.
//!
//! 4. Codegen
//!
//! The intermediate representation is converted to an initialization function which can be called
//! from Julia to initialize all exported items.

pub mod ast;
pub mod codegen;
pub mod ir;
pub mod model;
