/// The reason some node in an internal representation is `Invalid`.
///
/// For robustness reasons, passess in the compiler will often replace chunks of
/// the source code with a simple `Invalid` node whenever an error of any kind
/// occurs. These nodes are allowed to be processed by a code generator, and so
/// they should contain a rough reason behind their existence (such that the
/// generated code can throw a reasonably helpful error if the node is ever
/// executed).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Reason {
    SyntaxError,
}
