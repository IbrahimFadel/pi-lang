#include "parser.h"

std::vector<std::unique_ptr<Node>> parse_tokens(std::vector<std::shared_ptr<Token>> tokens)
{
    std::vector<std::unique_ptr<Node>> nodes;

    tok_pointer = 0;
    toks = tokens;
    cur_tok = toks[tok_pointer];

    binop_precedence["<"] = 10;
    binop_precedence["+"] = 20;
    binop_precedence["-"] = 20;
    binop_precedence["*"] = 40;

    while (cur_tok->type != Token_Type::tok_eof)
    {
        auto node = parse_token(cur_tok);
        nodes.push_back(std::move(node));
    }

    return nodes;
}

unique_ptr<Node> parse_token(std::shared_ptr<Token> token)
{
    switch (token->type)
    {
    case Token_Type::tok_fn:
        return parse_fn_declaration();
    case Token_Type::tok_if:
        return parse_if();
    case Token_Type::tok_import:
        return parse_import();
    case Token_Type::tok_for:
        return parse_for();
    case Token_Type::tok_i64:
        return parse_variable_declaration();
    case Token_Type::tok_i32:
        return parse_variable_declaration();
    case Token_Type::tok_i16:
        return parse_variable_declaration();
    case Token_Type::tok_i8:
        return parse_variable_declaration();
    case Token_Type::tok_float:
        return parse_variable_declaration();
    case Token_Type::tok_double:
        return parse_variable_declaration();
    case Token_Type::tok_string:
        return parse_variable_declaration();
    case Token_Type::tok_identifier:
        return parse_identifier_expression();
    default:
        return nullptr;
    }
}

unique_ptr<For_Node> parse_for()
{

    get_next_token(); //? eat 'for'

    throw_if_cur_tok_not_type(Token_Type::tok_open_paren, "Expected '(' in if statement", cur_tok->row, cur_tok->col);

    get_next_token(); //? eat '('

    auto variable = parse_variable_declaration();
    auto condition = parse_expression();
    std::string name = cur_tok->value;
    get_next_token(); //? eat variable name

    auto assignment = parse_variable_assignment(name, false);

    throw_if_cur_tok_not_type(Token_Type::tok_close_paren, "Expected ')' in for loop", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat ')'

    throw_if_cur_tok_not_type(Token_Type::tok_open_curly_bracket, "Expected '{' in for loop", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '{'

    auto then = parse_then();

    throw_if_cur_tok_not_type(Token_Type::tok_close_curly_bracket, "Expected '}' in for loop", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '}'

    return std::make_unique<For_Node>(std::move(variable), std::move(condition), std::move(assignment), std::move(then));
}

unique_ptr<Import_Node> parse_import()
{
    get_next_token(); //? eat 'import'

    throw_if_cur_tok_not_type(Token_Type::tok_string_lit, "Expected a string literal in import statement", cur_tok->row, cur_tok->col);

    std::string path = cur_tok->value;

    get_next_token(); //? eat string/path to file

    throw_if_cur_tok_not_type(Token_Type::tok_semicolon, "Expected a semicolon in import statement", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat ';'

    return std::make_unique<Import_Node>(path);
}

unique_ptr<If_Node> parse_if()
{
    get_next_token(); //? eat 'if'
    throw_if_cur_tok_not_type(Token_Type::tok_open_paren, "Expected '(' in if statement", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '('

    auto if_condition = parse_condition(Token_Type::tok_close_paren);
    auto conditions = std::get<0>(std::move(if_condition));
    auto condition_separators = std::get<1>(std::move(if_condition));

    throw_if_cur_tok_not_type(Token_Type::tok_close_paren, "Expected ')' in if statement", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat ')'
    throw_if_cur_tok_not_type(Token_Type::tok_open_curly_bracket, "Expected '{' in if statement", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '{'

    auto then = parse_then();
    if (!then)
        error("Error parsing function then block", cur_tok->row, cur_tok->col);

    throw_if_cur_tok_not_type(Token_Type::tok_close_curly_bracket, "Expected '}' in then block", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '}'

    return std::make_unique<If_Node>(std::move(conditions), condition_separators, std::move(then));
}

std::tuple<std::vector<std::unique_ptr<Condition_Node>>, std::vector<Token_Type>> parse_condition(Token_Type end_token)
{
    std::vector<std::unique_ptr<Condition_Node>> conditions;
    std::vector<Token_Type> condition_separators;

    while (cur_tok->type != end_token)
    {
        auto lhs = parse_expression(false);
        auto op = cur_tok->type;
        get_next_token(); //? eat operator
        auto rhs = parse_expression(false);

        auto condition = std::make_unique<Condition_Node>(std::move(lhs), op, std::move(rhs));
        conditions.push_back(std::move(condition));

        if (cur_tok->type == Token_Type::tok_and || cur_tok->type == Token_Type::tok_or)
        {
            condition_separators.push_back(cur_tok->type);
            get_next_token();
        }
    }

    return std::make_tuple(std::move(conditions), condition_separators);
}

unique_ptr<Variable_Declaration_Node> parse_variable_declaration()
{
    Variable_Type variable_type = token_type_to_variable_type(cur_tok->type);

    get_next_token(); //? eat 'i64', 'i32', 'float' or whatever

    std::string variable_name = cur_tok->value;

    get_next_token(); //? eat variable name
    get_next_token(); //? eat '='

    auto variable_value = parse_expression();

    return std::make_unique<Variable_Declaration_Node>(variable_name, variable_type, std::move(variable_value));
}

unique_ptr<Function_Node> parse_fn_declaration()
{
    get_next_token(); //? eat 'fn'

    auto prototype = parse_fn_prototype();
    if (!prototype)
        error("Error parsing function prototype", cur_tok->row, cur_tok->col);

    auto then = parse_then();
    if (!then)
        error("Error parsing function then block", cur_tok->row, cur_tok->col);

    throw_if_cur_tok_not_type(Token_Type::tok_close_curly_bracket, "Expected '}' in then block", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '}'

    auto node = std::make_unique<Function_Node>(std::move(prototype), std::move(then));
    node->set_node_type(Node_Type::FunctionNode);
    return node;
}

unique_ptr<Then_Node> parse_then()
{
    std::vector<std::unique_ptr<Node>> nodes;

    throw_if_cur_tok_is_type(Token_Type::tok_eof, "Unexpected EOF", cur_tok->row, cur_tok->col);

    while (cur_tok->type != Token_Type::tok_close_curly_bracket)
    {
        auto node = parse_token(cur_tok);
        nodes.push_back(std::move(node));
    }

    return std::make_unique<Then_Node>(std::move(nodes));
}

unique_ptr<Prototype_Node> parse_fn_prototype()
{
    throw_if_cur_tok_not_type(Token_Type::tok_identifier, "Expected function name in prototype", cur_tok->row, cur_tok->col);

    std::string fn_name = cur_tok->value;

    get_next_token(); //? eat function name
    throw_if_cur_tok_not_type(Token_Type::tok_open_paren, "Expected '(' in prototype", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '('

    std::vector<Variable_Type> param_types;
    std::vector<std::string> param_names;

    int param_counter = 0;
    while (cur_tok->type != Token_Type::tok_close_paren)
    {
        if (param_counter == 0)
        {
            param_types.push_back(token_type_to_variable_type(cur_tok->type));
        }
        else if (param_counter == 1)
        {
            param_names.push_back(cur_tok->value);
        }
        else if (param_counter == 2)
        {
            if (cur_tok->type == Token_Type::tok_comma)
            {
                param_counter = -1;
            }
        }

        get_next_token();
        param_counter++;
    }

    if (cur_tok->type != Token_Type::tok_close_paren)
        error("Expected ')' in prototype", cur_tok->row, cur_tok->col);

    get_next_token(); //? eat ')'
    throw_if_cur_tok_not_type(Token_Type::tok_arrow, "Expected '->' to indicate return type in prototype", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '->'

    Variable_Type return_type = token_type_to_variable_type(cur_tok->type);

    get_next_token(); //? eat return type
    throw_if_cur_tok_not_type(Token_Type::tok_open_curly_bracket, "Expected '{'", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '{'

    return std::make_unique<Prototype_Node>(fn_name, param_types, param_names, return_type);
}

unique_ptr<Expression_Node> parse_expression(bool needs_semicolon)
{
    auto lhs = parse_primary();
    if (!lhs)
        error("Error parsing primary", UNKOWN_LINE, UNKNOWN_COLUMN);

    auto binop_node = parse_binop_rhs(0, std::move(lhs));

    if (needs_semicolon)
        get_next_token(); //? eat ';'
    return binop_node;
}

unique_ptr<Expression_Node> parse_primary()
{
    switch (cur_tok->type)
    {
    case Token_Type::tok_identifier:
        return parse_identifier_expression();
    case Token_Type::tok_number:
        return parse_number_expression();
    default:
        break;
    }

    return nullptr;
}

unique_ptr<Expression_Node> parse_binop_rhs(int expression_precedence, unique_ptr<Expression_Node> lhs)
{
    while (true)
    {
        int tok_precedence = get_token_precedence();
        if (tok_precedence < expression_precedence)
        {
            return lhs;
        }

        std::string binop = cur_tok->value;

        get_next_token(); //? eat operator

        auto rhs = parse_primary();
        if (!rhs)
            error("Error parsing binary operator right hand side", last_tok->row, last_tok->col);

        int next_precedence = get_token_precedence();
        if (tok_precedence < next_precedence)
        {
            rhs = parse_binop_rhs(tok_precedence + 1, std::move(rhs));
            if (!rhs)
                error("Error parsing binary operator right hand side", last_tok->row, last_tok->col);
        }

        lhs = std::make_unique<Binary_Operation_Expression_Node>(binop, std::move(lhs), std::move(rhs));
    }

    return nullptr;
}

unique_ptr<Expression_Node> parse_number_expression()
{
    auto number_expression = std::make_unique<Number_Expression_Node>(std::stod(cur_tok->value));
    get_next_token(); //? eat number
    return std::move(number_expression);
}

unique_ptr<Expression_Node> parse_identifier_expression()
{
    std::string id_name = cur_tok->value;
    get_next_token(); //? eat identifier

    if (cur_tok->type == Token_Type::tok_open_paren)
    {
        return parse_function_call_node(id_name);
    }
    else if (cur_tok->type == Token_Type::tok_eq)
    {
        return parse_variable_assignment(id_name);
    }
    else
    {
        return std::make_unique<Variable_Reference_Node>(id_name);
    }
}

unique_ptr<Variable_Assignment_Node> parse_variable_assignment(std::string name, bool needs_semicolon)
{
    throw_if_cur_tok_not_type(Token_Type::tok_eq, "Expected '=' in variable assignment", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '='
    auto v = parse_expression(needs_semicolon);

    return std::make_unique<Variable_Assignment_Node>(name, std::move(v));
}

unique_ptr<Function_Call_Node> parse_function_call_node(std::string name)
{
    get_next_token(); //? eat '('

    std::vector<std::unique_ptr<Expression_Node>> args;
    if (cur_tok->type != Token_Type::tok_close_paren)
    {
        while (true)
        {
            if (auto arg = parse_expression(false))
            {
                args.push_back(std::move(arg));
            }
            else
            {
                error("Error parsing function call parameters", cur_tok->row, cur_tok->col);
                return nullptr;
            }

            if (cur_tok->type == Token_Type::tok_close_paren)
                break;
            if (cur_tok->type != Token_Type::tok_comma)
                error("Expected ')' or ',' in argument list", cur_tok->row, cur_tok->col);

            get_next_token();
        }
    }

    throw_if_cur_tok_not_type(Token_Type::tok_close_paren, "Expected ')' at end of function call", cur_tok->row, cur_tok->col);

    get_next_token(); //? eat ')'

    return std::make_unique<Function_Call_Node>(name, std::move(args));
}

void throw_if_cur_tok_is_type(Token_Type type, const char *msg, int line, int position)
{
    if (cur_tok->type == type)
        error(msg, line, position);
}

void throw_if_cur_tok_not_type(Token_Type type, const char *msg, int line, int position)
{
    if (cur_tok->type != type)
        error(msg, line, position);
}

Variable_Type token_type_to_variable_type(Token_Type type)
{
    switch (type)
    {
    case Token_Type::tok_i64:
        return Variable_Type::type_i64;
    case Token_Type::tok_i32:
        return Variable_Type::type_i32;
    case Token_Type::tok_i16:
        return Variable_Type::type_i16;
    case Token_Type::tok_i8:
        return Variable_Type::type_i8;
    case Token_Type::tok_float:
        return Variable_Type::type_float;
    case Token_Type::tok_double:
        return Variable_Type::type_double;
    case Token_Type::tok_string:
        return Variable_Type::type_string;
    case Token_Type::tok_bool:
        return Variable_Type::type_bool;
    case Token_Type::tok_toi64:
        return Variable_Type::type_i64;
    case Token_Type::tok_toi32:
        return Variable_Type::type_i32;
    case Token_Type::tok_toi16:
        return Variable_Type::type_i16;
    case Token_Type::tok_toi8:
        return Variable_Type::type_i8;
    case Token_Type::tok_identifier:
        return Variable_Type::type_object;
    default:
        error("Could not convert token type to variable type", UNKOWN_LINE, UNKNOWN_COLUMN);
        break;
    }
    return Variable_Type::type_null;
}

void get_next_token()
{
    tok_pointer++;
    cur_tok = toks[tok_pointer];
    last_tok = toks[tok_pointer - 1];
}

int get_token_precedence()
{
    int tok_precedence = binop_precedence[cur_tok->value];
    if (tok_precedence <= 0)
        return -1;
    return tok_precedence;
}