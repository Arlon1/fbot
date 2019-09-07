import sqlite3

from botpackage.helper import helper

_botname = 'Hannah'

def processMessage(args, rawMessage, db_connection):
    message = rawMessage['message'].lower()
    name = rawMessage['name'].lower()
	
    if ('<3' in message) and (not 'hannah' in message) and ('ludwig' in name):
        return helper.botMessage(":(", _botname)
