from botpackage.helper import helper
from botpackage.helper.mystrip import _space_chars, norm

_botname = "	 	  	    			  	    	 Jörn"
_bottrigger = 'praise'

_last = ["fbot",None] # save the two last persons

def processMessage(args, rawMessage, db_connection):
    global _last

    if len(args) < 1 or args[0].lower() != "!" + _bottrigger:
        # remember a person
        now = rawMessage["name"].strip(_space_chars)
        if now != _last[0]:
            _last = [now, _last[0]]
        return

    if len(args) < 2:
        now = rawMessage["name"].strip(_space_chars)
        if now != _last[0]:
            target = _last[0] # prev person
        else:
            target = _last[1] # prevprev person
    else:
        target = args[1].strip(_space_chars) # a specified person

    return helper.botMessage(f'praise the {target} \o/', _botname) # praise the fbot \o/
