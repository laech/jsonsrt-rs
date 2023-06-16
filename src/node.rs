#[derive(Debug, PartialEq)]
pub enum Node<'a> {
  Object(Vec<(&'a str, Node<'a>)>),
  Array(Vec<Node<'a>>),
  Value(&'a str),
}
