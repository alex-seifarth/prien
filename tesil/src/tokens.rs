use util::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum IntegerBase {
    Binary,
    Octal,
    Decimal,
    Hexadecimal
}

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    EndOfFile,
    LeftParen(utf8::Position),      // '('
    RightParen(utf8::Position),     // ')'
    LeftBrace(utf8::Position),      // '{'
    RightBrace(utf8::Position),     // '}'
    LeftBracket(utf8::Position),    // '['
    RightBracket(utf8::Position),   // ']'
    Star(utf8::Position),           // '*'
    Minus(utf8::Position),          // '-'
    Plus(utf8::Position),           // '+'
    Slash(utf8::Position),          // '/'
    Assign(utf8::Position),         // '='
    Ampersand(utf8::Position),      // '&'
    Vert(utf8::Position),           // '|'
    Tilde(utf8::Position),          // '~'
    ExclamationMark(utf8::Position),// '!'
    Caret(utf8::Position),          // '^'
    Less(utf8::Position),           // '<'
    Greater(utf8::Position),        // '>'
    Colon(utf8::Position),          // ':'
    Semicolon(utf8::Position),      // ';'
    Comma(utf8::Position),          // ','
    Dot(utf8::Position),            // '.'
    Hash(utf8::Position),           // '#'

    LessThan(utf8::Position),       // '<='
    GreaterThan(utf8::Position),    // '>='
    Implies(utf8::Position),        // '=>'
    AddAssign(utf8::Position),      // '+='
    SubAssign(utf8::Position),      // '-='
    MulAssign(utf8::Position),      // '*='
    DivAssign(utf8::Position),      // '/='
    AndAssign(utf8::Position),      // '&='
    OrAssign(utf8::Position),       // '|='
    EXorAssign(utf8::Position),     // '^='
    LogicAnd(utf8::Position),       // '&&'
    LogicOr(utf8::Position),        // '||'
    RightArrow(utf8::Position),     // '->'
    LeftArrow(utf8::Position),      // '<-'
    Range(utf8::Position),          // '..'
    ScopeSep(utf8::Position),       // '::'
    Equals(utf8::Position),         // '=='
    ShiftRight(utf8::Position),     // '>>'
    ShiftLeft(utf8::Position),      // '<<'

    // Identifier string
    // [_a-zA-Z][_a-zA-Z0-9]*
    Identifier {
        start: utf8::Position,
        end: utf8::Position,
        source: String,
    },

    // double slash comment // until end of line
    Comment {
        start: utf8::Position,
        comment: String,
    },

    // Integer literal (unsigned)
    // Decimal: ([0-9]('[0-9])?)+
    // Binary: (0b|0B) ([01] ('[01])?)+
    // Hexadecimal: (0x|0X) ([0-9a-fA-F] ('[0-9a-fA-F])?)+
    // Octal: <not supported>
    Integer {
        start: utf8::Position,
        end: utf8::Position,
        source: String,
        value: u64,
        base: IntegerBase,
    },

    FloatNumber {
        start: utf8::Position,
        end: utf8::Position,
        source: String,
        value: f64,
    },

    // ("[^"]*")+
    String {
        start: utf8::Position,
        end: utf8::Position,
        source: String,
    },

    KwImport(utf8::Position),       // 'import'
    KwTypeI8(utf8::Position),       // 'i8'
    KwTypeI16(utf8::Position),      // 'i16'
    KwTypeI32(utf8::Position),      // 'i32'
    KwTypeI64(utf8::Position),      // 'i64'
    KwTypeU8(utf8::Position),       // 'u8'
    KwTypeU16(utf8::Position),      // 'u16'
    KwTypeU32(utf8::Position),      // 'u32'
    KwTypeU64(utf8::Position),      // 'u64'
    KwTypeBool(utf8::Position),     // 'bool'
    KwTypeF32(utf8::Position),      // 'f32'
    KwTypeF64(utf8::Position),      // 'f64'
    KwTypeChar(utf8::Position),     // 'char'
    KwFn(utf8::Position),           // 'fn'
    KwStruct(utf8::Position),       // 'struct'
    KwEnum(utf8::Position),         // 'enum'
    KwType(utf8::Position),         // 'type'
    KwBreak(utf8::Position),        // 'break'
    KwContinue(utf8::Position),     // 'continue'
    KwExpect(utf8::Position),       // 'expect'
    KwLet(utf8::Position),          // 'let'
    KwMut(utf8::Position),          // 'mut'
}