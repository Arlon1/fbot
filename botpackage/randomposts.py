import random

from botpackage.helper import helper

_randompost_one_over_probability = 500
_names = {
	'manf':['Hack the planet!','Paladschinken'],
	'luise':['Lalalalalalalalalalala','Luise'],
	'kathie':['Ich bin voll putzig. Du auch.','Kathie'],
	'nicholas':['Knuddeln ist auch ein Sport. *knuddelt*','Nicholas']
}


def processMessage(args, rawMessage, db_connection):
	if random.randrange(_randompost_one_over_probability) == 0:
		msg_obj = _names[random.choice(list(_names))]
		return helper.botMessage(msg_obj[0], msg_obj[1])
	return
