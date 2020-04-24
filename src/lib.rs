use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while, take_while1},
    character::complete::{alphanumeric0, char, one_of, space0, space1},
    combinator::{map_res, not, opt},
    error::ParseError,
    multi::{count, many0, many1, many_till, separated_list},
    Err, IResult,
};

#[derive(PartialEq, Clone, Debug)]
pub struct Model {
    pub predicate_items: Vec<PredicateItem>,
    pub par_decl_items: Vec<ParDeclItem>,
    pub var_decl_items: Vec<VarDeclItem>,
    pub constraint_items: Vec<ConstraintItem>,
    pub solve_item: SolveItem,
}
pub fn model<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Model, E> {
    let (input, predicate_items) = many0(predicate_item)(input)?;
    // let (input, predicate_items) = separated_list(tag(";"), predicate_item)(input)?;
    let (input, par_decl_items) = many0(par_decl_item)(input)?;
    // let (input, par_decl_items) = separated_list(tag(";"), par_decl_item)(input)?;
    let (input, var_decl_items) = many0(var_decl_item)(input)?;
    // let (input, var_decl_items) = separated_list(tag(";"), var_decl_item)(input)?;
    let (input, constraint_items) = many0(constraint_item)(input)?;
    let (input, solve_item) = solve_item(input)?;
    Ok((
        input,
        Model {
            predicate_items,
            par_decl_items,
            var_decl_items,
            constraint_items,
            solve_item,
        },
    ))
}
#[derive(PartialEq, Clone, Debug)]
pub struct PredicateItem {
    ident: String,
    parameters: Vec<(PredParamType, String)>,
}
pub fn predicate_item<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, PredicateItem, E> {
    let (input, _) = tag("predicate")(input)?;
    let (input, _) = space1(input)?;
    let (input, ident) = identifier(input)?;
    let (input, _) = char('(')(input)?;
    let (input, parameters) = separated_list(tag(","), pred_param_type_ident_pair)(input)?;
    let (input, _) = char(')')(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(';')(input)?;
    let (input, _) = char('\n')(input)?; // CHECK
    Ok((input, PredicateItem { ident, parameters }))
}
pub fn pred_param_type_ident_pair<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, (PredParamType, String), E> {
    let (input, pred_param_type) = pred_param_type(input)?;
    let (input, _) = char(':')(input)?;
    let (input, ident) = identifier(input)?;
    Ok((input, (pred_param_type, ident)))
}

pub fn identifier<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, String, E> {
    let (input, first) = one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")(input)?;
    let (input, rest) = take_while(is_identifier_rest)(input)?;
    Ok((input, format!("{}{}", first, rest)))
}
#[derive(PartialEq, Clone, Debug)]
pub enum BasicParType {
    Bool,
    Int,
    Float,
    SetOfInt,
}
pub fn basic_par_type<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicParType, E> {
    let (input, string) = alt((tag("bool"), tag("int"), tag("float"), tag("set of int")))(input)?;
    match string {
        "bool" => Ok((input, BasicParType::Bool)),
        "int" => Ok((input, BasicParType::Int)),
        "float" => Ok((input, BasicParType::Float)),
        "set of int" => Ok((input, BasicParType::SetOfInt)),
        x => panic!("unmatched basic par type {}", x),
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum ParType {
    BasicParType(BasicParType),
    Array(IndexSet, BasicParType),
}
pub fn par_type<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, ParType, E> {
    let (input, par_type) = alt((pt_basic_par_type, array_par_type))(input)?;
    Ok((input, par_type))
}
pub fn pt_basic_par_type<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, ParType, E> {
    let (input, par_type) = basic_par_type(input)?;
    Ok((input, ParType::BasicParType(par_type)))
}
pub fn array_par_type<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, ParType, E> {
    let (input, _) = tag("array")(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = char('[')(input)?;
    let (input, _) = space0(input)?;
    let (input, index_set) = index_set(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(']')(input)?;
    let (input, _) = space1(input)?;
    let (input, _tag) = tag("of")(input)?;
    let (input, _) = space1(input)?;
    let (input, par_type) = basic_par_type(input)?;
    Ok((input, ParType::Array(index_set, par_type)))
}
#[derive(PartialEq, Clone, Debug)]
pub enum BasicVarType {
    Bool,
    Int,
    Float,
    Domain(Domain),
}
pub fn basic_var_type<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicVarType, E> {
    dbg!("in basic_var_type");
    let (input, _tag) = tag("var")(input)?;
    let (input, _) = space1(input)?;
    dbg!(_tag);
    let (input, bvt) = alt((bvt_bool, bvt_int, bvt_float, bvt_domain))(input)?;
    dbg!(&bvt);
    Ok((input, bvt))
}
pub fn bvt_bool<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, BasicVarType, E> {
    let (input, _tag) = tag("bool")(input)?;
    Ok((input, BasicVarType::Bool))
}
pub fn bvt_int<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, BasicVarType, E> {
    let (input, _tag) = tag("int")(input)?;
    Ok((input, BasicVarType::Int))
}
pub fn bvt_float<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, BasicVarType, E> {
    let (input, _tag) = tag("float")(input)?;
    Ok((input, BasicVarType::Float))
}
pub fn bvt_domain<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, BasicVarType, E> {
    let (input, domain) = domain(input)?;
    Ok((input, BasicVarType::Domain(domain)))
}
// introduced by me used in basic-var-type and basic-pred-param-type
#[derive(PartialEq, Clone, Debug)]
pub enum Domain {
    IntRange(i128, i128),
    FloatRange(f64, f64),
    SetIntNonEmpty(Vec<i128>),
    SetIntRange(i128, i128),
    SetInt(Vec<i128>), // possibly empty
}
pub fn domain<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Domain, E> {
    dbg!("in domain");
    let (input, domain) = alt((
        int_range,
        float_range,
        set_of_int_range,
        set_of_ints,
        set_of_ints_non_empty,
    ))(input)?;
    dbg!(&domain);
    Ok((input, domain))
}
pub fn int_range<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Domain, E> {
    let (input, lb) = int_literal(input)?;
    let (input, _tag) = tag("..")(input)?;
    let (input, ub) = int_literal(input)?;
    Ok((input, Domain::IntRange(lb, ub)))
}
pub fn float_range<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Domain, E> {
    let (input, lb) = float_literal(input)?;
    let (input, _tag) = tag("..")(input)?;
    let (input, ub) = float_literal(input)?;
    Ok((input, Domain::FloatRange(lb, ub)))
}
// "set" "of" <int_literal> ".." <int_literal>
pub fn set_of_int_range<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Domain, E> {
    let (input, _tag) = tag("set of")(input)?;
    let (input, lb) = int_literal(input)?;
    let (input, _tag) = tag("..")(input)?;
    let (input, ub) = int_literal(input)?;
    Ok((input, Domain::SetIntRange(lb, ub)))
}
// "set" "of" "{" [ <int-literal> "," ... ] "}"
pub fn set_of_ints<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Domain, E> {
    let (input, _tag) = tag("set of {")(input)?;
    let (input, v) = separated_list(tag(","), int_literal)(input)?;
    let (input, _tag) = tag("}")(input)?;
    Ok((input, Domain::SetInt(v)))
}
// "{" <int-literal> "," ... "}"
pub fn set_of_ints_non_empty<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Domain, E> {
    let (input, _) = char('{')(input)?;
    let (input, v) = separated_list(tag(","), int_literal)(input)?;
    let (input, _) = char('}')(input)?;
    Ok((input, Domain::SetIntNonEmpty(v)))
}
#[derive(PartialEq, Clone, Debug)]
pub struct ArrayVarType {
    index_set: IndexSet,
    basic_var_type: BasicVarType,
}
pub fn array_var_type<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, ArrayVarType, E> {
    let (input, _tag) = tag("array")(input)?;
    let (input, _) = char('[')(input)?;
    let (input, index_set) = index_set(input)?;
    let (input, _) = char(']')(input)?;
    let (input, _tag) = tag("of")(input)?;
    let (input, basic_var_type) = basic_var_type(input)?;
    Ok((
        input,
        ArrayVarType {
            index_set,
            basic_var_type,
        },
    ))
}
#[derive(PartialEq, Clone, Debug)]
pub struct IndexSet(i128);
pub fn index_set<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, IndexSet, E> {
    let (input, _tag) = tag("1")(input)?;
    let (input, _tag) = tag("..")(input)?;
    let (input, int) = int_literal(input)?;
    Ok((input, IndexSet(int)))
}
#[derive(PartialEq, Clone, Debug)]
pub enum BasicPredParamType {
    BasicParType(BasicParType),
    BasicVarType(BasicVarType),
    Domain(Domain),
    VarSetOFInt, // shouldn't this be a basic-var-type
}
pub fn basic_pred_param_type<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicPredParamType, E> {
    let (input, bppt) = alt((
        bppt_basic_par_type,
        bppt_basic_var_type,
        bppt_domain,
        var_set_of_int, // TODO remove if "var set of int" is in <basic_var_type>
    ))(input)?;
    Ok((input, bppt))
}
pub fn bppt_basic_par_type<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicPredParamType, E> {
    let (input, bpt) = basic_par_type(input)?;
    Ok((input, BasicPredParamType::BasicParType(bpt)))
}
pub fn bppt_basic_var_type<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicPredParamType, E> {
    let (input, bvt) = basic_var_type(input)?;
    Ok((input, BasicPredParamType::BasicVarType(bvt)))
}
pub fn bppt_domain<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicPredParamType, E> {
    let (input, domain) = domain(input)?;
    Ok((input, BasicPredParamType::Domain(domain)))
}
// "var" "set" "of" "int"
// shouldn't this be a basic-var-type basic-par-type
pub fn var_set_of_int<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicPredParamType, E> {
    let (input, _tag) = tag("var set of int")(input)?;
    Ok((input, BasicPredParamType::VarSetOFInt))
}
#[derive(PartialEq, Clone, Debug)]
pub enum PredParamType {
    BasicPredParamType(BasicPredParamType),
    Array(PredIndexSet, BasicPredParamType),
}
pub fn pred_param_type<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, PredParamType, E> {
    let (input, ppt) = alt((ppt_basic_pred_param_type, array_of_pred_index_set))(input)?;
    Ok((input, ppt))
}
pub fn ppt_basic_pred_param_type<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, PredParamType, E> {
    let (input, bppt) = basic_pred_param_type(input)?;
    Ok((input, PredParamType::BasicPredParamType(bppt)))
}
pub fn array_of_pred_index_set<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, PredParamType, E> {
    let (input, _tag) = tag("array")(input)?;
    let (input, _space) = space1(input)?;
    let (input, _) = char('[')(input)?;
    let (input, _) = char(']')(input)?;
    let (input, pis) = pred_index_set(input)?;
    let (input, _space) = space1(input)?;
    let (input, _tag) = tag("of")(input)?;
    let (input, _space) = space1(input)?;
    let (input, bppt) = basic_pred_param_type(input)?;
    Ok((input, PredParamType::Array(pis, bppt)))
}
#[derive(PartialEq, Clone, Debug)]
pub enum PredIndexSet {
    IndexSet(IndexSet),
    Int,
}
pub fn pred_index_set<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, PredIndexSet, E> {
    let (input, index_set) = alt((pis_int, pis_index_set))(input)?;
    Ok((input, index_set))
}
pub fn pis_int<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, PredIndexSet, E> {
    let (input, _tag) = tag("int")(input)?;
    Ok((input, PredIndexSet::Int))
}
pub fn pis_index_set<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, PredIndexSet, E> {
    let (input, is) = index_set(input)?;
    Ok((input, PredIndexSet::IndexSet(is)))
}
#[derive(PartialEq, Clone, Debug)]
pub enum BasicLiteralExpr {
    BoolLiteral(bool),
    IntLiteral(i128),
    FloatLiteral(f64),
    SetLiteral(SetLiteral),
}
pub fn basic_literal_expr<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicLiteralExpr, E> {
    let (input, ble) = alt((
        ble_bool_literal,
        ble_int_literal,
        ble_float_literal,
        ble_set_literal,
    ))(input)?;
    Ok((input, ble))
}
pub fn ble_bool_literal<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicLiteralExpr, E> {
    let (input, expr) = bool_literal(input)?;
    Ok((input, BasicLiteralExpr::BoolLiteral(expr)))
}
pub fn ble_int_literal<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicLiteralExpr, E> {
    let (input, expr) = int_literal(input)?;
    Ok((input, BasicLiteralExpr::IntLiteral(expr)))
}
pub fn ble_float_literal<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicLiteralExpr, E> {
    let (input, expr) = float_literal(input)?;
    Ok((input, BasicLiteralExpr::FloatLiteral(expr)))
}
pub fn ble_set_literal<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicLiteralExpr, E> {
    let (input, expr) = set_literal(input)?;
    Ok((input, BasicLiteralExpr::SetLiteral(expr)))
}

#[derive(PartialEq, Clone, Debug)]
pub enum BasicExpr {
    BasicLiteralExpr(BasicLiteralExpr),
    VarParIdentifier(String),
}
pub fn basic_expr<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, BasicExpr, E> {
    let (input, expr) = alt((be_basic_literal_expr, be_var_par_identifier))(input)?;
    Ok((input, expr))
}
pub fn be_basic_literal_expr<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicExpr, E> {
    let (input, expr) = basic_literal_expr(input)?;
    Ok((input, BasicExpr::BasicLiteralExpr(expr)))
}
pub fn be_var_par_identifier<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, BasicExpr, E> {
    let (input, expr) = var_par_identifier(input)?;
    Ok((input, BasicExpr::VarParIdentifier(expr)))
}

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    BasicExpr(BasicExpr),
    ArrayLiteral(ArrayLiteral),
}
pub fn expr<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Expr, E> {
    let (input, expr) = alt((e_basic_expr, e_array_literal))(input)?;
    Ok((input, expr))
}
pub fn e_basic_expr<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Expr, E> {
    let (input, basic_expr) = basic_expr(input)?;
    Ok((input, Expr::BasicExpr(basic_expr)))
}
pub fn e_array_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Expr, E> {
    let (input, array_literal) = array_literal(input)?;
    Ok((input, Expr::ArrayLiteral(array_literal)))
}
#[derive(PartialEq, Clone, Debug)]
pub enum ParExpr {
    BasicLiteralExpr(BasicLiteralExpr),
    ParArrayLiteral(ParArrayLiteral),
}
pub fn par_expr<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, ParExpr, E> {
    println!("Hi par_expr");
    let (input, expr) = alt((pe_basic_literal_expr, pe_par_array_literal))(input)?;
    println!("Bye par_expr {:?}", expr);
    Ok((input, expr))
}
pub fn pe_basic_literal_expr<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, ParExpr, E> {
    let (input, expr) = basic_literal_expr(input)?;
    Ok((input, ParExpr::BasicLiteralExpr(expr)))
}
pub fn pe_par_array_literal<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, ParExpr, E> {
    let (input, expr) = par_array_literal(input)?;
    Ok((input, ParExpr::ParArrayLiteral(expr)))
}
pub fn var_par_identifier<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, String, E> {
    let (input, first) = one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_")(input)?;
    let (input, rest) = take_while(is_identifier_rest)(input)?;
    Ok((input, format!("{}{}", first, rest)))
}
pub fn is_identifier_rest(c: char) -> bool {
    match c {
        'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l' | 'm' | 'n' | 'o'
        | 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'v' | 'w' | 'x' | 'y' | 'z' | 'A' | 'B' | 'C'
        | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O' | 'P' | 'Q'
        | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' | '_' | '0' | '1' | '2' | '3'
        | '4' | '5' | '6' | '7' | '8' | '9' => true, //one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ_0123456789")(input.into()) {
        _ => false,
    }
}
pub fn bool_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, bool, E> {
    let (input, string) = alt((tag("true"), tag("false")))(input)?;
    match string {
        "true" => Ok((input, true)),
        "false" => Ok((input, false)),
        x => panic!("unmatched bool literal {}", x),
    }
}
pub fn int_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, i128, E> {
    // TODO support hexadecimals and octals
    let (input, negation) = opt(char('-'))(input)?;
    let (input, int) = map_res(take_while1(is_dec_digit), from_dec)(input)?;
    if negation.is_some() {
        Ok((input, -(int as i128)))
    } else {
        Ok((input, int as i128))
    }
}
fn from_dec(input: &str) -> Result<u128, std::num::ParseIntError> {
    u128::from_str_radix(input, 10)
}
fn is_dec_digit(c: char) -> bool {
    c.is_digit(10)
}
pub fn float_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, f64, E> {
    // TODO
    let (input, _tag) = tag("TODO")(input)?;
    Ok((input, 0.0))
}
#[derive(PartialEq, Clone, Debug)]
pub enum SetLiteral {
    IntRange(i128, i128),
    FloatRange(f64, f64),
    SetFloats(Vec<f64>),
    SetInts(Vec<i128>), // possibly empty
}
pub fn set_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, SetLiteral, E> {
    let (input, sl) = alt((
        sl_int_range,
        sl_float_range,
        sl_set_of_floats,
        sl_set_of_ints,
    ))(input)?;
    Ok((input, sl))
}
pub fn sl_int_range<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, SetLiteral, E> {
    let (input, lb) = int_literal(input)?;
    let (input, _tag) = tag("..")(input)?;
    let (input, ub) = int_literal(input)?;
    Ok((input, SetLiteral::IntRange(lb, ub)))
}
pub fn sl_float_range<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, SetLiteral, E> {
    let (input, lb) = float_literal(input)?;
    let (input, _tag) = tag("..")(input)?;
    let (input, ub) = float_literal(input)?;
    Ok((input, SetLiteral::FloatRange(lb, ub)))
}
// "{" <int-literal> "," ... "}"
pub fn sl_set_of_ints<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, SetLiteral, E> {
    let (input, _) = char('{')(input)?;
    let (input, v) = separated_list(tag(","), int_literal)(input)?;
    let (input, _) = char('}')(input)?;
    Ok((input, SetLiteral::SetInts(v)))
}
// "{" <float-literal> "," ... "}"
pub fn sl_set_of_floats<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, SetLiteral, E> {
    let (input, _tag) = tag("{")(input)?;
    let (input, v) = separated_list(tag(","), float_literal)(input)?;
    let (input, _tag) = tag("}")(input)?;
    Ok((input, SetLiteral::SetFloats(v)))
}
type ArrayLiteral = Vec<BasicExpr>;
pub fn array_literal<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, ArrayLiteral, E> {
    let (input, _) = char('[')(input)?;
    let (input, al) = separated_list(tag(","), basic_expr)(input)?;
    let (input, _) = char(']')(input)?;
    Ok((input, al))
}
type ParArrayLiteral = Vec<BasicLiteralExpr>;
pub fn par_array_literal<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, ParArrayLiteral, E> {
    print!("hi par_array_literal");
    let (input, _) = char('[')(input)?;
    let (input, v) = separated_list(tag(","), basic_literal_expr)(input)?;
    let (input, _) = char(']')(input)?;
    Ok((input, v))
}
#[derive(PartialEq, Clone, Debug)]
pub struct ParDeclItem {
    parameter_type: ParType,
    identifier: String,
    expr: ParExpr,
}
pub fn par_decl_item<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, ParDeclItem, E> {
    let (input, parameter_type) = par_type(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _tag) = space1(input)?;
    let (input, identifier) = var_par_identifier(input)?;
    println!("identifier {}", identifier);
    let (input, _) = space1(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _tag) = space1(input)?;
    let (input, expr) = par_expr(input)?;
    let (input, _tag) = space0(input)?;
    let (input, _) = char(';')(input)?;
    let (input, _tag) = space0(input)?;
    let (input, _) = char('\n')(input)?;
    Ok((
        input,
        ParDeclItem {
            parameter_type,
            identifier,
            expr,
        },
    ))
}
#[derive(PartialEq, Clone, Debug)]
pub enum VarDeclItem {
    Basic(BasicVarType, String, Annotations, Option<BasicExpr>),
    Array(ArrayVarType, String, Annotations, ArrayLiteral),
}
pub fn var_decl_item<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, VarDeclItem, E> {
    dbg!("in var_decl_item");
    let (input, vdi) = alt((vdi_basic_var, vdi_array))(input)?;
    let (input, _tag) = space0(input)?;
    let (input, _) = char(';')(input)?;
    let (input, _tag) = space0(input)?;
    let (input, _) = char('\n')(input)?;
    dbg!(&vdi);
    Ok((input, vdi))
}
pub fn vdi_basic_var<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, VarDeclItem, E> {
    dbg!("in vdi_basic_var");
    let (input, bvt) = basic_var_type(input)?;
    dbg!(&bvt);
    let (input, _) = space0(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space0(input)?;
    let (input, vpi) = var_par_identifier(input)?;
    dbg!(&vpi);
    let (input, _) = space0(input)?;
    let (input, anno) = annotations(input)?;
    dbg!(&anno);
    let (input, _) = space0(input)?;
    let (input, _) = opt(char('='))(input)?;
    let (input, _) = space0(input)?;
    let (input, be) = opt(basic_expr)(input)?;
    dbg!(&be);
    Ok((input, VarDeclItem::Basic(bvt, vpi, anno, be)))
}
pub fn vdi_array<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, VarDeclItem, E> {
    let (input, avt) = array_var_type(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space0(input)?;
    let (input, vpi) = var_par_identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, anno) = annotations(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, al) = array_literal(input)?;
    Ok((input, VarDeclItem::Array(avt, vpi, anno, al)))
}
#[derive(PartialEq, Clone, Debug)]
pub struct ConstraintItem {
    ident: String,
    exprs: Vec<Expr>,
    annos: Vec<Annotation>,
}
pub fn constraint_item<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, ConstraintItem, E> {
    let (input, _tag) = tag("constraint")(input)?;
    let (input, _) = space1(input)?;
    let (input, ident) = identifier(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = space0(input)?;
    let (input, exprs) = separated_list(tag(","), expr)(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(')')(input)?;
    let (input, _) = space0(input)?;
    let (input, annos) = annotations(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(';')(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('\n')(input)?;
    Ok((
        input,
        ConstraintItem {
            ident,
            exprs,
            annos,
        },
    ))
}
#[derive(PartialEq, Clone, Debug)]
pub struct SolveItem;
pub fn solve_item<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, SolveItem, E> {
    // TODO
    Ok((input, SolveItem))
}
pub fn _annotation<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Annotation, E> {
    let (input, _mark) = tag("::")(input)?;
    let (input, annotation) = annotation(input)?;
    Ok((input, annotation))
}
type Annotations = Vec<Annotation>;
pub fn annotations<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Annotations, E> {
    dbg!("in annotations");
    let (input, annos) = many0(annotation)(input)?;
    Ok((input, annos))
}
type Annotation = String;
pub fn annotation<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Annotation, E> {
    dbg!("in annotation");
    let (input, _) = tag("::")(input)?;
    let (input, _) = space0(input)?;
    let (input, anno) = identifier(input)?;
    dbg!(&anno);
    let (input, _) = space0(input)?;
    Ok((input, anno))
}
pub struct AnnExpr;
pub fn ann_expr<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, AnnExpr, E> {
    // TODO
    Ok((input, AnnExpr))
}
