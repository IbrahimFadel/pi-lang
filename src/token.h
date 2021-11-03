#ifndef TOKEN_H
#define TOKEN_H

typedef struct Position {
  int line, col;
} Position;

typedef enum TokenType {
  TOKTYPE_ILLEGAL,
  TOKTYPE_EOF,

  TOKTYPE_IDENT,

  TOKTYPE_BASIC_LIT_BEGIN,

  TOKTYPE_INT,
  TOKTYPE_FLOAT,
  TOKTYPE_STRING_LIT,
  TOKTYPE_CHAR_LIT,

  TOKTYPE_BASIC_LIT_END,

  TOKTYPE_PACKAGE,
  TOKTYPE_PUB,
  TOKTYPE_FN,
  TOKTYPE_RETURN,
  TOKTYPE_MUT,
  TOKTYPE_TYPE,
  TOKTYPE_INTERFACE,
  TOKTYPE_STRUCT,
  TOKTYPE_NIL,
  TOKTYPE_IF,
  TOKTYPE_ELSE,
  TOKTYPE_SIZEOF,

  TOKTYPE_LPAREN,
  TOKTYPE_RPAREN,
  TOKTYPE_LBRACE,
  TOKTYPE_RBRACE,
  TOKTYPE_LBRACKET,
  TOKTYPE_RBRACKET,
  TOKTYPE_SEMICOLON,
  TOKTYPE_PERIOD,
  TOKTYPE_ARROW,
  TOKTYPE_COMMA,

  TOKTYPE_PLUS,
  TOKTYPE_MINUS,
  TOKTYPE_ASTERISK,
  TOKTYPE_SLASH,
  TOKTYPE_EQ,
  TOKTYPE_AMPERSAND,
  TOKTYPE_PIPE,

  TOKTYPE_CMP_AND,
  TOKTYPE_CMP_OR,
  TOKTYPE_CMP_LT,
  TOKTYPE_CMP_GT,
  TOKTYPE_CMP_LTE,
  TOKTYPE_CMP_GTE,
  TOKTYPE_CMP_EQ,
  TOKTYPE_CMP_NEQ,

  TOKTYPE_TYPES_BEGIN,

  TOKTYPE_ARRAY,

  TOKTYPE_I64,
  TOKTYPE_I32,
  TOKTYPE_I16,
  TOKTYPE_I8,

  TOKTYPE_U64,
  TOKTYPE_U32,
  TOKTYPE_U16,
  TOKTYPE_U8,

  TOKTYPE_F64,
  TOKTYPE_F32,

  TOKTYPE_TYPES_END,
} TokenType;

typedef struct Token {
  Position pos;
  TokenType type;
  const char *value;
} Token;

TokenType lookup_keyword(const char *str);
void token_destroy(Token *tok);

#endif