use nom::{
  branch::alt,
  bytes::complete::{tag, take_while1},
  multi::{fold_many0, separated_list0},
  sequence::separated_pair,
  IResult,
};

enum Jsonish {
  Object(Vec<(Vec<u8>, Jsonish)>),
  Array(Vec<Jsonish>),
  Value(Vec<u8>),
}

fn jsonish<'a>() -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Jsonish> {
  ws(alt((object(), array(), value())))
}

fn array() -> impl Fn(&[u8]) -> IResult<&[u8], Jsonish> {
  |input| {
    let (input, _) = ws(tag("["))(input)?;
    let (input, values) = separated_list0(ws(tag(",")), jsonish())(input)?;
    let (input, _) = ws(tag("]"))(input)?;
    Ok((input, Jsonish::Array(values)))
  }
}

fn object() -> impl Fn(&[u8]) -> IResult<&[u8], Jsonish> {
  |input| {
    let (input, _) = ws(tag("{"))(input)?;
    let (input, values) = separated_list0(ws(tag(",")), member())(input)?;
    let (input, _) = ws(tag("}"))(input)?;
    Ok((input, Jsonish::Object(values)))
  }
}

fn member<'a>() -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], (Vec<u8>, Jsonish)> {
  separated_pair(value_content(), ws(tag(":")), jsonish())
}

fn value() -> impl FnMut(&[u8]) -> IResult<&[u8], Jsonish> {
  |input| {
    let (input, value) = value_content()(input)?;
    Ok((input, Jsonish::Value(value)))
  }
}

fn value_content() -> impl Fn(&[u8]) -> IResult<&[u8], Vec<u8>> {
  |input| alt((string(), non_string()))(input)
}

fn non_string() -> impl Fn(&[u8]) -> IResult<&[u8], Vec<u8>> {
  |input| {
    let (input, value) =
      take_while1(|x| !(x as char).is_whitespace() && !b",:{}[]".contains(&x))(input)?;
    Ok((input, value.to_owned()))
  }
}

fn string() -> impl Fn(&[u8]) -> IResult<&[u8], Vec<u8>> {
  |input| {
    let (input, a) = tag("\"")(input)?;
    let (input, mut result) = fold_many0(
      alt((
        take_while1(|x| !b"\\\"".contains(&x)),
        tag("\\\""),
        take_while1(|x| x != b'\"'),
      )),
      || a.to_owned(),
      |mut acc: Vec<_>, xs| {
        acc.extend_from_slice(xs);
        acc
      },
    )(input)?;
    let (input, b) = tag("\"")(input)?;
    result.extend_from_slice(b);
    Ok((input, result))
  }
}

fn ws<'a, O>(
  mut parse: impl FnMut(&'a [u8]) -> IResult<&'a [u8], O>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O> {
  move |input| {
    let (input, _) = take_while1(|b| (b as char).is_whitespace())(input)?;
    let (input, output) = parse(input)?;
    let (input, _) = take_while1(|b| (b as char).is_whitespace())(input)?;
    Ok((input, output))
  }
}
