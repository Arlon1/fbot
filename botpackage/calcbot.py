import botpackage.helper.calc as calc
import botpackage.helper.timeout as timeout
from botpackage.helper import helper

_botname = 'calc'
_help = '%s <mathematischer Ausdruck>'%_botname

def processMessage(args, rawMessage, db_connection):
    if len(args) < 1 or args[0].lower() != "!" + _botname:
        return

    expr = rawMessage["message"].strip()[len("!" + _botname):]

    ret = timeout.timed_run(calc.evaluate, [expr])

    if isinstance(ret, Exception):
        return helper.botMessage("Ein Fehler trat auf: %s"%ret, _botname)
    elif ret is None:
        return helper.botMessage("Die Evaluierung dauerte zu lange.", _botname)
    else:
        return helper.botMessage("Das Ergebnis ist %s."%ret, _botname)

    
