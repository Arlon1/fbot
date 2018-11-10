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

parser = pyparsing.infixNotation(constant, _operators)

def evaluate(s):
    try:
        return parser.parseString(s, parseAll=True)[0].eval()
    except pyparsing.ParseException as e:
        return e
    except RecursionError as e:
        return e

