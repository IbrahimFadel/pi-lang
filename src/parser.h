#ifndef PARSER_H
#define PARSER_H

#include <iostream>
#include <vector>
#include <memory>
#include <variant>
#include <llvm/IR/Value.h>

#include "lexer.h"

using std::cout;
using std::endl;

enum Node_Types
{
  VariableDeclarationNode,
  ConstantDeclarationNode,
  FunctionDeclarationNode,
  FunctionCallNode,
  NumberExpressionNode,
  StringExpressionNode,
  BinaryOperationNode
};

enum Variable_Scope
{
  Global,
  Function
};

enum Expression_Types
{
  NumberExpression,
  StringExpression
};

enum Variable_Types
{
  IntType,
  StringType,
  VoidType,
  FloatType,
  ArrayType
};

struct Number_Expression_Node
{
  int type = 0;
  std::vector<std::string> numbers;
  std::vector<std::string> ops;
};

struct String_Expression_Node
{
};

struct Expression_Node
{
  Expression_Types type;
  std::variant<Number_Expression_Node, String_Expression_Node> number_expression, string_expression;
};

struct Binary_Operation_Node
{
  Expression_Node *left;
  std::string op;
  Expression_Node *right;
};

struct Constant_Declaration_Node
{
  std::string name;
  Variable_Types type;
  Variable_Scope scope;
  std::unique_ptr<Expression_Node> expression;
  llvm::Value *code_gen();
};

struct Variable_Declaration_Node
{
  std::string name;
  Variable_Types type;
  Variable_Scope scope;
  std::unique_ptr<Expression_Node> expression;
};

struct Node
{
  Node_Types type;
  std::variant<Constant_Declaration_Node *, Variable_Declaration_Node> constant_declaration_node, variable_declaration_node;
};

std::vector<Node *> parse_tokens(std::vector<std::shared_ptr<Token>>);

Constant_Declaration_Node *create_constant_declaration_node(std::vector<std::shared_ptr<Token>>, int);
std::unique_ptr<Expression_Node> create_expression_node(std::vector<std::shared_ptr<Token>>, int);

void print_nodes(std::vector<Node *>);

#endif