import random

from botpackage.helper import helper

_randompost_one_over_probability = 500
_names = [
	['Hack the planet!','Paladschinken'],
	['Lalalalalalalalalalala','Luise'],
	['Ich bin voll putzig. Du auch.','Kathie'],
	['Knuddeln ist auch ein Sport. *knuddelt*','Nicholas'],
	['Praise the Jana \o/', '\t \t  \t    \t\t\t  \t    \t Jörn']
]


def processMessage(args, rawMessage, db_connection):
	if random.randrange(_randompost_one_over_probability) == 0:
		msg_obj = random.choice(_names)
		return helper.botMessage(msg_obj[0], msg_obj[1])
	return
