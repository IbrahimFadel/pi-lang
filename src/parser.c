#include "parser.h"

#include <stdio.h>
#include <stdlib.h>

ParseContext *parsecontext_create(Token *toks) {
  ParseContext *ctx = malloc(sizeof(ParseContext));
  ctx->toks = toks;
  ctx->tok_ptr = 0;
  ctx->cur_tok = ctx->toks[ctx->tok_ptr];
  ctx->functions = NULL;
  ctx->types = NULL;
  // there is a better way im just stupid
  ctx->tok_precedence_map[0].type = TOKTYPE_EQ;
  ctx->tok_precedence_map[0].prec = 2;
  ctx->tok_precedence_map[1].type = TOKTYPE_CMP_AND;
  ctx->tok_precedence_map[1].prec = 3;
  ctx->tok_precedence_map[2].type = TOKTYPE_CMP_OR;
  ctx->tok_precedence_map[2].prec = 5;
  ctx->tok_precedence_map[3].type = TOKTYPE_CMP_LT;
  ctx->tok_precedence_map[3].prec = 10;
  ctx->tok_precedence_map[4].type = TOKTYPE_CMP_GT;
  ctx->tok_precedence_map[4].prec = 10;
  ctx->tok_precedence_map[5].type = TOKTYPE_CMP_LTE;
  ctx->tok_precedence_map[5].prec = 10;
  ctx->tok_precedence_map[6].type = TOKTYPE_CMP_GTE;
  ctx->tok_precedence_map[6].prec = 10;
  ctx->tok_precedence_map[7].type = TOKTYPE_CMP_EQ;
  ctx->tok_precedence_map[7].prec = 10;
  ctx->tok_precedence_map[8].type = TOKTYPE_CMP_NEQ;
  ctx->tok_precedence_map[8].prec = 10;
  ctx->tok_precedence_map[9].type = TOKTYPE_PLUS;
  ctx->tok_precedence_map[9].prec = 20;
  ctx->tok_precedence_map[10].type = TOKTYPE_MINUS;
  ctx->tok_precedence_map[10].prec = 20;
  ctx->tok_precedence_map[11].type = TOKTYPE_ASTERISK;
  ctx->tok_precedence_map[11].prec = 40;
  ctx->tok_precedence_map[12].type = TOKTYPE_SLASH;
  ctx->tok_precedence_map[12].prec = 40;
  ctx->tok_precedence_map[13].type = TOKTYPE_PERIOD;
  ctx->tok_precedence_map[13].prec = 50;
  ctx->tok_precedence_map[14].type = TOKTYPE_ARROW;
  ctx->tok_precedence_map[14].prec = 50;
  return ctx;
}

void parser_fatal(const char *msg) {
  printf("%s\n", msg);
  exit(1);
}

Token parser_eat(ParseContext *ctx) {
  ctx->tok_ptr++;
  ctx->cur_tok = ctx->toks[ctx->tok_ptr];
  return ctx->cur_tok;
}

Token parser_expect(ParseContext *ctx, TokenType type, const char *msg) {
  if (ctx->cur_tok.type != type) {
    parser_fatal(msg);
  }
  Token tok = ctx->cur_tok;
  parser_eat(ctx);
  return tok;
}

Token parser_expect_range(ParseContext *ctx, TokenType begin, TokenType end, const char *msg) {
  if (ctx->cur_tok.type <= begin || ctx->cur_tok.type >= end)
    parser_fatal(msg);
  Token tok = ctx->cur_tok;
  parser_eat(ctx);
  return tok;
}

int parser_get_tokprec(ParseContext *ctx, TokenType tok) {
  int i;
  for (i = 0; i < sizeof(ctx->tok_precedence_map) / sizeof(ctx->tok_precedence_map[0]); i++) {
    if (tok == ctx->tok_precedence_map[i].type) return ctx->tok_precedence_map[i].prec;
  }
  return -1;
}

const char *parse_pkg(ParseContext *ctx) {
  parser_expect(ctx, TOKTYPE_PACKAGE, "expected 'pkg' in package statement");
  return parser_expect(ctx, TOKTYPE_IDENT, "expected identifier following 'pkg'").value;
}

FnDecl *parse_fn_decl(ParseContext *ctx, bool pub) {
  FnDecl *fn = malloc(sizeof(FnDecl));
  parser_expect(ctx, TOKTYPE_FN, "expected 'fn' in function declaration");

  fn->receiver = NULL;
  if (ctx->cur_tok.type == TOKTYPE_LPAREN) {
    fn->receiver = parse_fn_receiver(ctx);
  }

  fn->name = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier in function name").value;
  parser_expect(ctx, TOKTYPE_LPAREN, "expected '(' before function parameter listing");

  fn->params = parse_paramlist(ctx);
  parser_expect(ctx, TOKTYPE_RPAREN, "expected ')' after function parameter listing");

  fn->return_type = malloc(sizeof *fn->return_type);
  fn->return_type->type = EXPRTYPE_VOID;
  if (ctx->cur_tok.type == TOKTYPE_ARROW) {
    parser_eat(ctx);
    fn->return_type = parse_type_expr(ctx);
  }

  fn->body = parse_block_stmt(ctx);

  fn->pub = pub;
  return fn;
}

BlockStmt *parse_block_stmt(ParseContext *ctx) {
  BlockStmt *block = malloc(sizeof *block);
  parser_expect(ctx, TOKTYPE_LBRACE, "expected '{' after function parameter listing");
  block->stmts = NULL;
  while (ctx->cur_tok.type != TOKTYPE_RBRACE)
    cvector_push_back(block->stmts, *parse_stmt(ctx));
  parser_expect(ctx, TOKTYPE_RBRACE, "expected '}' after function body");
  block->variables = NULL;
  return block;
}

FnReceiver *parse_fn_receiver(ParseContext *ctx) {
  FnReceiver *recv = malloc(sizeof(FnReceiver));
  parser_expect(ctx, TOKTYPE_LPAREN, "expected '(' in function receiver");
  recv->type = parse_type_expr(ctx);
  recv->name = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier after receiver type").value;
  parser_expect(ctx, TOKTYPE_RPAREN, "expected ')' in function receiver");
  return recv;
}

cvector_vector_type(Param) parse_paramlist(ParseContext *ctx) {
  cvector_vector_type(Param) paramlist = NULL;
  while (ctx->cur_tok.type != TOKTYPE_RPAREN) {
    Param *p = parse_param(ctx);
    cvector_push_back(paramlist, *p);
    if (ctx->cur_tok.type == TOKTYPE_COMMA) {
      parser_eat(ctx);
    } else if (ctx->cur_tok.type != TOKTYPE_RPAREN) {
      parser_fatal("expected ')' at end of param list");
    }
  }
  return paramlist;
}

Param *parse_param(ParseContext *ctx) {
  Param *p = malloc(sizeof(Param));
  p->mut = false;
  if (ctx->cur_tok.type == TOKTYPE_MUT) {
    p->mut = true;
    parser_eat(ctx);
  }
  p->type = parse_type_expr(ctx);
  p->name = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier in parameter").value;
  return p;
}

Expr *parse_type_expr(ParseContext *ctx) {
  if (ctx->cur_tok.type > TOKTYPE_TYPES_BEGIN && ctx->cur_tok.type < TOKTYPE_TYPES_END)
    return parse_primitive_type_expr(ctx);

  switch (ctx->cur_tok.type) {
    case TOKTYPE_INTERFACE:
      return parse_interface_type_expr(ctx);
    case TOKTYPE_STRUCT:
      return parse_struct_type_expr(ctx);
    case TOKTYPE_IDENT: {
      Expr *e = parse_ident_expr(ctx);
      if (ctx->cur_tok.type == TOKTYPE_ASTERISK) {
        e->type = EXPRTYPE_PTR;
        e->value.pointer_type->pointer_to_type = e;
        parser_eat(ctx);
        while (ctx->cur_tok.type == TOKTYPE_ASTERISK) {
          e->value.pointer_type->pointer_to_type = e;
          parser_eat(ctx);
        }
      }
      return e;
    }
    default:
      parser_fatal("unimplemented type expression");
  }
  return NULL;
}

Expr *parse_struct_type_expr(ParseContext *ctx) {
  parser_expect(ctx, TOKTYPE_STRUCT, "expected 'struct' in struct type expression");
  parser_expect(ctx, TOKTYPE_LBRACE, "expected '{' in struct type expression");

  Expr *s = malloc(sizeof *s);
  s->type = EXPRTYPE_STRUCT;
  s->value.struct_type = malloc(sizeof *s->value.struct_type);
  s->value.struct_type->properties = NULL;
  s->value.struct_type->interface_implementations = NULL;
  while (ctx->cur_tok.type != TOKTYPE_RBRACE) {
    cvector_push_back(s->value.struct_type->properties, *parse_property(ctx));
    parser_expect(ctx, TOKTYPE_SEMICOLON, "expected ';' after property in struct type property list");
    if (ctx->cur_tok.type != TOKTYPE_RBRACE && ctx->cur_tok.type != TOKTYPE_IDENT) {
      parser_fatal("expected a property or '}' in struct type property list");
    }
  }

  parser_expect(ctx, TOKTYPE_RBRACE, "expected '}' in struct type expression");
  return s;
}

Property *parse_property(ParseContext *ctx) {
  Property *p = malloc(sizeof *p);
  p->pub = false;
  p->mut = false;
  if (ctx->cur_tok.type == TOKTYPE_PUB) {
    p->pub = true;
    parser_eat(ctx);
  }
  if (ctx->cur_tok.type == TOKTYPE_MUT) {
    p->mut = true;
    parser_eat(ctx);
  }
  p->type = parse_type_expr(ctx);
  p->names = NULL;
  while (ctx->cur_tok.type != TOKTYPE_SEMICOLON) {
    cvector_push_back(p->names, parser_expect(ctx, TOKTYPE_IDENT, "expected identifier in property identifier list").value);
    if (ctx->cur_tok.type == TOKTYPE_COMMA) {
      parser_eat(ctx);
    } else if (ctx->cur_tok.type != TOKTYPE_SEMICOLON) {
      parser_fatal("expected ';' at end of property");
    }
  }

  return p;
}

Expr *parse_interface_type_expr(ParseContext *ctx) {
  parser_expect(ctx, TOKTYPE_INTERFACE, "expected 'interface' in interface type expression");
  parser_expect(ctx, TOKTYPE_LBRACE, "expected '{' in interface type expression");

  Expr *interface = malloc(sizeof *interface);
  interface->type = EXPRTYPE_INTERFACE;
  interface->value.interface_type = malloc(sizeof *interface->value.interface_type);
  interface->value.interface_type->methods = NULL;
  while (ctx->cur_tok.type != TOKTYPE_RBRACE) {
    cvector_push_back(interface->value.interface_type->methods, *parse_method_decl(ctx));
    parser_expect(ctx, TOKTYPE_SEMICOLON, "expected ';' after method in interface type method list");
    if (ctx->cur_tok.type != TOKTYPE_RBRACE && ctx->cur_tok.type != TOKTYPE_IDENT) {
      parser_fatal("expected a method or '}' in interface type method list");
    }
  }

  parser_expect(ctx, TOKTYPE_RBRACE, "expected '}' at end of interface type expression");

  return interface;
}

Method *parse_method_decl(ParseContext *ctx) {
  Method *m = malloc(sizeof *m);
  m->pub = false;
  if (ctx->cur_tok.type == TOKTYPE_PUB) {
    m->pub = true;
    parser_eat(ctx);
  }
  m->name = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier in method declaration").value;
  parser_expect(ctx, TOKTYPE_LPAREN, "expected '(' before method param list");
  m->params = parse_paramlist(ctx);
  parser_expect(ctx, TOKTYPE_RPAREN, "expected ')' after method param list");

  m->return_type = malloc(sizeof *m->return_type);
  m->return_type->type = EXPRTYPE_VOID;
  if (ctx->cur_tok.type == TOKTYPE_ARROW) {
    parser_eat(ctx);
    m->return_type = parse_type_expr(ctx);
  }

  return m;
}

Expr *parse_primitive_type_expr(ParseContext *ctx) {
  Expr *expr = malloc(sizeof(Expr));
  TokenType ty = parser_expect_range(ctx, TOKTYPE_TYPES_BEGIN, TOKTYPE_TYPES_END, "expected a type in primitive type expression").type;

  expr->type = EXPRTYPE_PRIMITIVE;
  expr->value.primitive_type = malloc(sizeof *expr->value.primitive_type);
  expr->value.primitive_type->type = ty;
  if (ctx->cur_tok.type != TOKTYPE_ASTERISK) {
    return (Expr *)expr;
  }

  expr->type = EXPRTYPE_PTR;
  expr->value.pointer_type->pointer_to_type = expr;
  parser_eat(ctx);
  while (ctx->cur_tok.type == TOKTYPE_ASTERISK) {
    expr->value.pointer_type->pointer_to_type = expr;
    parser_eat(ctx);
  }
  return expr;
}

Stmt *parse_stmt(ParseContext *ctx) {
  if (ctx->cur_tok.type > TOKTYPE_TYPES_BEGIN && ctx->cur_tok.type < TOKTYPE_TYPES_END)
    return parse_var_decl(ctx, false, false);
  switch (ctx->cur_tok.type) {
    case TOKTYPE_MUT:
      parser_eat(ctx);
      return parse_var_decl(ctx, false, true);
    case TOKTYPE_RETURN:
      return parse_return_stmt(ctx);
    case TOKTYPE_IDENT:
      switch (ctx->toks[ctx->tok_ptr + 1].type) {
        case TOKTYPE_LPAREN:
          parser_fatal("function calls unimplemented");
        case TOKTYPE_IDENT:
          return parse_var_decl(ctx, false, false);
        default:
          parser_fatal("unknown token when parsing statement");
      }
      break;
    default:
      parser_fatal("unknown token when parsing statement");
      break;
  }
  return NULL;
}

Stmt *parse_return_stmt(ParseContext *ctx) {
  parser_expect(ctx, TOKTYPE_RETURN, "expected 'return' in return statement");
  Stmt *ret = malloc(sizeof *ret);
  ret->type = STMTTYPE_RETURN;

  ret->value.ret = malloc(sizeof *ret->value.ret);
  ret->value.ret->v = malloc(sizeof *ret->value.ret);
  ret->value.ret->v->type = EXPRTYPE_VOID;
  if (ctx->cur_tok.type != TOKTYPE_SEMICOLON)
    ret->value.ret->v = parse_expr(ctx);

  parser_expect(ctx, TOKTYPE_SEMICOLON, "expected ';' after return value");

  return ret;
}

Stmt *parse_var_decl(ParseContext *ctx, bool pub, bool mut) {
  Stmt *var = malloc(sizeof *var);
  var->type = STMTTYPE_VARDECL;
  var->value.var_decl = malloc(sizeof *var->value.var_decl);
  var->value.var_decl->pub = pub;
  var->value.var_decl->mut = mut;
  var->value.var_decl->type = parse_type_expr(ctx);
  var->value.var_decl->names = NULL;
  var->value.var_decl->values = NULL;
  while (ctx->cur_tok.type != TOKTYPE_EQ && ctx->cur_tok.type != TOKTYPE_SEMICOLON) {
    const char *ident = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier in var decl").value;
    cvector_push_back(var->value.var_decl->names, ident);

    if (ctx->cur_tok.type == TOKTYPE_COMMA) {
      parser_eat(ctx);
    } else if (ctx->cur_tok.type != TOKTYPE_EQ && ctx->cur_tok.type != TOKTYPE_SEMICOLON) {
      parser_fatal("expected '=' or ';' after var decl ident list");
    }
  }

  if (ctx->cur_tok.type == TOKTYPE_SEMICOLON) {
    parser_eat(ctx);
    return var;
  }

  parser_expect(ctx, TOKTYPE_EQ, "expected '=' in var decl");

  int i = 0;
  while (ctx->cur_tok.type != TOKTYPE_SEMICOLON) {
    if (i + 1 > cvector_size(var->value.var_decl->names)) {
      parser_fatal("too many values in var decl");
    }
    Expr *v = parse_expr(ctx);
    cvector_push_back(var->value.var_decl->values, v);

    if (ctx->cur_tok.type == TOKTYPE_COMMA) {
      parser_eat(ctx);
    } else if (ctx->cur_tok.type != TOKTYPE_SEMICOLON) {
      parser_fatal("expected ';' after var decl value list");
    }
    i++;
  }

  if (i < cvector_size(var->value.var_decl->names) && i != 1) {
    parser_fatal("too few values in var decl");
  }

  parser_expect(ctx, TOKTYPE_SEMICOLON, "expected ';' in var decl");

  return var;
}

Expr *parse_expr(ParseContext *ctx) {
  return parse_binary_expr(ctx, 1);
}

Expr *parse_binary_expr(ParseContext *ctx, int prec1) {
  Expr *x = parse_unary_expr(ctx);
  while (true) {
    int oprec = parser_get_tokprec(ctx, ctx->cur_tok.type);
    TokenType op = ctx->cur_tok.type;

    if (oprec < prec1) {
      return x;
    }

    parser_eat(ctx);
    Expr *y = parse_binary_expr(ctx, oprec + 1);

    Expr *binop = malloc(sizeof *binop);
    binop->type = EXPRTYPE_BINARY;
    binop->value.binop = malloc(sizeof *binop->value.binop);
    binop->value.binop->x = x;
    binop->value.binop->op = op;
    binop->value.binop->y = y;
    x = binop;
  }
}

Expr *parse_unary_expr(ParseContext *ctx) {
  switch (ctx->cur_tok.type) {
    case TOKTYPE_AMPERSAND:
      parser_fatal("unimplemented unary expression");
      break;
    default:
      return parse_primary_expr(ctx);
  }
  return NULL;
}

Expr *parse_primary_expr(ParseContext *ctx) {
  switch (ctx->cur_tok.type) {
    case TOKTYPE_IDENT:
      return parse_ident_expr(ctx);
    case TOKTYPE_INT:
    case TOKTYPE_FLOAT:
    case TOKTYPE_STRING_LIT:
    case TOKTYPE_CHAR_LIT:
      return parse_basic_lit(ctx);
    default:
      parser_fatal("unimplemented primary expr");
      break;
  }
  return NULL;
}

Expr *parse_ident_expr(ParseContext *ctx) {
  const char *value = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier").value;
  Expr *ident = malloc(sizeof *ident);
  ident->type = EXPRTYPE_IDENT;
  ident->value.ident = malloc(sizeof *ident->value.ident);
  ident->value.ident->value = value;
  return ident;
}

Expr *parse_basic_lit(ParseContext *ctx) {
  Token tok = parser_expect_range(ctx, TOKTYPE_BASIC_LIT_BEGIN, TOKTYPE_BASIC_LIT_END, "expected literal");
  Expr *lit = malloc(sizeof *lit);
  lit->type = EXPRTYPE_BASIC_LIT;
  lit->value.basic_lit = malloc(sizeof *lit->value.basic_lit);
  lit->value.basic_lit->type = tok.type;
  switch (tok.type) {
    case TOKTYPE_INT:
      lit->value.basic_lit->value.int_lit = malloc(sizeof *lit->value.basic_lit->value.int_lit);
      lit->value.basic_lit->value.int_lit->bits = 32;
      lit->value.basic_lit->value.int_lit->is_signed = true;
      lit->value.basic_lit->value.int_lit->value = atoi(tok.value);
      break;
    case TOKTYPE_FLOAT:
      lit->value.basic_lit->value.float_lit = malloc(sizeof *lit->value.basic_lit->value.float_lit);
      lit->value.basic_lit->value.float_lit->bits = 32;
      lit->value.basic_lit->value.float_lit->value = atoi(tok.value);
      break;
    default:
      parser_fatal("unimplemented basic lit type");
  }
  return lit;
}

TypeDecl *parse_type_decl(ParseContext *ctx, bool pub) {
  parser_expect(ctx, TOKTYPE_TYPE, "expected 'type' in type declaration");
  TypeDecl *ty = malloc(sizeof *ty);
  ty->pub = pub;
  ty->name = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier in type declaration").value;
  ty->value = parse_type_expr(ctx);
  return ty;
}

void parse_pkg_file_tokens(ParseContext *ctx) {
  ctx->pkg = parse_pkg(ctx);
  int i = 0;
  while (ctx->cur_tok.type != TOKTYPE_EOF) {
    switch (ctx->cur_tok.type) {
      case TOKTYPE_PACKAGE:
        parser_fatal("cannot have more than one 'pkg' in file");
      case TOKTYPE_FN:
        cvector_push_back(ctx->functions, *parse_fn_decl(ctx, false));
        break;
      case TOKTYPE_PUB:
        parser_eat(ctx);
        switch (ctx->cur_tok.type) {
          case TOKTYPE_FN:
            cvector_push_back(ctx->functions, *parse_fn_decl(ctx, true));
            break;
          case TOKTYPE_TYPE:
            cvector_push_back(ctx->types, *parse_type_decl(ctx, true));
            break;
          default:
            break;
        }
        break;
      case TOKTYPE_TYPE:
        cvector_push_back(ctx->types, *parse_type_decl(ctx, false));
        break;
      default:
        parser_fatal("unimplemented token");
    }
    i++;
  }
}