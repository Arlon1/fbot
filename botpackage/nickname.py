import sqlite3

from botpackage.helper import helper
from botpackage.helper.mystrip import mystrip

_help = 'usage: !nickname <nickname> [[-a|-r] <nickname>]'
_botname = 'nicknamebot'
_max_nicks_pp = 25

def processMessage(args, rawMessage, db_connection):
	if len(args) == 0:
		return
	if args[0].lower() not in ['!nickname', '!nicknames']:
		return

	if len(args) < 2:
		return helper.botMessage(_help, _botname)

	cursor = db_connection.cursor()

	if args[1].lower() == 'self':
		args[1] = rawMessage['name']

	useridQuery = cursor.execute(
				'SELECT userid '
				'FROM nicknames '
				'WHERE lower(nickname) == ? '
			';', (args[1].lower(), )).fetchone()
	if useridQuery is None:
		return helper.botMessage('Ich kenne ' + args[1] + ' nicht.', _botname)

	userid = useridQuery[0]

	username = cursor.execute(
				'SELECT nickname '
				'FROM nicknames '
				'WHERE userid == ? '
				'AND deletable == 1'
				';', (userid, )).fetchone()[0]

	# print nicknames
	if len(args) == 2:
		message = ''
		nicknamelist = []
		for nickname in cursor.execute(
					'SELECT nickname '
					'FROM nicknames '
					'WHERE userid == ? '
					'AND deletable == 0'
					';', (userid, )
				):
			nicknamelist.append(nickname[0])

		if len(nicknamelist) < 1:
			return helper.botMessage(username + ' hat keine nicknames.', _botname)

		message = username + ' hat die Nicknames:\n'
		for nickname in nicknamelist:
			message += nickname + '\n'

	# add or remove nicknames
	elif len(args) == 4:
		if args[3].lower() in ['self'] or args[3] == '-':
			return helper.botMessage('Der nickname ' + args[3] + ' ist reserviert.', _botname)

		if args[2] == '-a': # add nickname
			cursor = db_connection.cursor() ## todo braucht man das wirklich
			toAddCheck = cursor.execute(
						'SELECT userid '
						'FROM nicknames '
						'WHERE lower(nickname) == ?'
						';', (args[3].lower(), )).fetchone()
			if toAddCheck is not None:
				message = 'Der nickname ' + args[3] + ' existiert schon.'
			else:
				nickListLength = cursor.execute(
							'SELECT COUNT(*) '
							'FROM nicknames '
							'WHERE userid = ?'
							';', (userid,)
						).fetchone()
				if nickListLength[0] >= _max_nicks_pp:
					return helper.botMessage(username + ' hat schon ' + str(_max_nicks_pp) + ' nicknames.', _botname)
				cursor.execute('INSERT INTO nicknames (nickname, userid) VALUES (?, ?);', (args[3], userid))
				db_connection.commit()
				return helper.botMessage(username + ' hat nun den Nickname ' + args[3] + '.', _botname)
		elif args[2] == '-r': # remove nickname
			toRemoveCheck = cursor.execute(
						'SELECT userid '
						'FROM nicknames '
						'WHERE lower(nickname) == ? '
						'AND deletable == 0 '
						';', (args[3].lower(), )).fetchone()
			if toRemoveCheck is None:
				message = 'Ich kenne ' + args[3] + ' nicht.'
			else:
				cursor = db_connection.cursor()
				cursor.execute(
						'DELETE FROM nicknames '
						'WHERE lower(nickname) == ?'
						';', (args[3].lower(), ))
				db_connection.commit()
				return helper.botMessage(username + ' hat jetzt den nicknamen ' + args[3] + ' nicht mehr.', _botname)
		else:
			return helper.botMessage(_help, _botname)
	else:
		return helper.botMessage(_help, _botname)

	return helper.botMessage(message, _botname)
