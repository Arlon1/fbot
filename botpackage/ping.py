import sqlite3
import parsedatetime, datetime

from botpackage.helper import helper
from botpackage.helper.mystrip import stripFromBegin, _space_chars
from botpackage.helper.split import split_with_quotation_marks

_botname = '                               Daniel                     '
_posts_since_ping = 25

def processMessage(args, rawMessage, db_connection):
	cursor = db_connection.cursor()

	message = None

	if rawMessage['username'] != None:
		recipientNicks = [rawMessage['username'].lower()]
	else:
		recipientNicks = [rawMessage['name'].lower()]
	for nick in cursor.execute(
				'SELECT lower(nickname) '
				'FROM nicknames '
				'WHERE userid = ('
					'SELECT userid '
					'FROM nicknames '
					'WHERE lower(nickname) == ? '
					'ORDER BY deletable DESC'
				');', (recipientNicks[0],)
		):
		if nick[0].lower() not in recipientNicks:
			recipientNicks.append(nick[0].lower())


	pingProperties = dict(print=True, delete=True)
	for nick in recipientNicks:
		cursor = db_connection.cursor()
		for pong in \
					cursor.execute(
						'SELECT sender, message, messageid '
						'FROM pings '
						'WHERE lower(recipient) == ? '
						';', (nick.lower(), )
				):
			if pong[2] + _posts_since_ping > rawMessage['id']:
				pingProperties['print'] = False
			# ~ pongSplit = split_with_quotation_marks(pong[1])
			# ~ if len(pongSplit) >= 3 \
					# ~ and pongSplit[0].startswith('-') \
					# ~ and pongSplit[0][1:] == 'pong':
				# ~ pongTime = datetime.datetime(*parsedatetime.Calendar().parse(pongSplit[1])[0][:6])
				# ~ if datetime.datetime.now() < pongTime:
					# ~ pingProperties['print'] = False
				# ~ else:
					# ~ pingProperties['delete'] = False

			if pingProperties['print'] == True:
				if message == None:
					message = rawMessage['name'] + ', dir wollte jemand etwas sagen:'
				message += '\n' + pong[0] + ' sagte: ' + pong[1]
			else:
				pingProperties['ping'] = True
		# ~ if delete und so
		cursor.execute(
					'DELETE '
					'FROM pings '
					'WHERE LOWER(recipient) == ? '
					';', (nick,))
		# ~ db_connection.commit()

	if len(args) >= 2 and args[0] == '!ping' and ''.join(args[2:]).strip(''.join(_space_chars)) != '':
		cursor = db_connection.cursor()
		pingCount = cursor.execute(
					'SELECT count(*) '
					'FROM pings '
					'WHERE recipient == ?'
					';', (args[1], )
		).fetchone()
		if pingCount[0] is 0:
			cursor.execute(
						'INSERT OR REPLACE '
						'INTO pings '
						'(recipient, message, sender, messageid) '
						'VALUES (?, ?, ?, ?)'
						';', (
							args[1],
							stripFromBegin(rawMessage['message'], args[0:2]),
							rawMessage['name'],
							rawMessage['id'],
						)
				)
		else:
			cursor.execute(
						'UPDATE pings '
						'SET message = ?, messageid = ? '
						'WHERE recipient = ? '
						'AND sender = ?'
						';', (
							''.join(x + ' ' for x in args[2:]).strip(),
							rawMessage['id'],
							args[1],
							rawMessage['name'],
						)
				)
		db_connection.commit()

	if message is not None:
		return helper.botMessage(message, _botname)
