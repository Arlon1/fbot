import math
import pyparsing


def _unary_eval(chr_to_func):
    class c:
        def __init__(self, token):
            op, self.value = token[0]
            self.func = chr_to_func[op]

        def eval(self):
            return self.func(self.value)

    return c

def _binary_eval(chr_to_func, ass):
    class c:
        def __init__(self, token):
            if ass == _leftas:
                value = iter(token[0])
            else:
                value = iter(token[0][::-1])

            self.list = [next(value)]
            for op, val in zip(value, value):
                self.list.append((chr_to_func[op], val))

        def eval(self):
            ret = self.list[0].eval()
            for i in self.list[1:]:
                ret = i[0](ret, i[1].eval())
            return ret

    return c

_functions = {
        "sqrt": {"call": math.sqrt, "max_args": 1, "min_args" : 1},
        "gcd": {"call": math.gcd, "max_args": 2, "min_args": 2}
        }

class _func_eval:
    @pyparsing.traceParseAction
    def __init__(self, line, pos, token):
        f = _functions[token[0]]
        self.call = f["call"]
        if len(token[1]) < f["min_args"]:
            raise pyparsing.ParseFatalException("To few arguments (%s) for function %s, expected at least %s."%(len(token[1]), token[0], f["min_args"]), pos)
        if len(token[1]) > f["max_args"]:
            raise pyparsing.ParseFatalException("To many arguments (%s) for function %s, expected at most %s."%(len(token[1]), token[0], f["max_args"]), pos)

        self.values = token[1]

    def eval(self):
        return self.call(*[i.eval() for i in self.values])



_leftas = pyparsing.opAssoc.LEFT
_rightas = pyparsing.opAssoc.RIGHT

_function = pyparsing.Forward()

_mul = lambda x,y : x*y
_div = lambda x,y : x/y
_add = lambda x,y : x+y
_sub = lambda x,y : x-y
_mod = lambda x,y : x%y

_operators = [
        ("-", 1, _rightas, _unary_eval({"-" : lambda x: -x})),
        ("^", 2, _rightas, _binary_eval({"^": pow}, _rightas)),
        (pyparsing.oneOf("* /"), 2, _leftas, _binary_eval({"*": _mul, "/": _div}, _leftas)),
        (pyparsing.oneOf("+ -"), 2, _leftas, _binary_eval({"+": _add, "-": _sub}, _leftas)),
        ("%", 2, _leftas, _binary_eval({"%": _mod}, _leftas))]

integer = pyparsing.pyparsing_common.signed_integer
floating = pyparsing.pyparsing_common.fnumber

constant = integer | floating

class _evalConst:
    def __init__(self, token):
        self.value = token[0]

    def eval(self):
        return self.value

constant.setParseAction(_evalConst)

parser = pyparsing.infixNotation(constant | _function, _operators)

_function << (pyparsing.Or(map(pyparsing.Keyword, _functions.keys())) + pyparsing.Suppress("(") +  pyparsing.Group(pyparsing.delimitedList(parser)) + pyparsing.Suppress(")"))
_function.setParseAction(_func_eval)

def parse(s):
    return parser.parseString(s, parseAll=True)

def try_parse(s):
    try:
        return parse(s)
    except pyparsing.ParseException as e:
        return e
    except pyparsing.ParseFatalException as e:
        return e
    except RecursionError as e:
        return e

def evaluate(s):
    try:
        return parse(s)[0].eval()
    except pyparsing.ParseException as e:
        return e
    except pyparsing.ParseFatalException as e:
        return e
    except RecursionError as e:
        return e

