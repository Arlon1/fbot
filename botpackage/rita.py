import re
import hashlib
import random

import botpackage.helper.argparse as argparse # ~ import argparse
from botpackage.helper import helper, ud
from botpackage.helper.mystrip import mystrip, stripFromBegin

_botname = 'Dr. Ritastein'
_bottrigger = 'rita'.lower()
_usageTemplate = 'usage: !' + _bottrigger + ' '
_help = _usageTemplate + '[decide|ping|sing|ud] [args]'
_help_sing = _usageTemplate + 'sing [song [-l|--learn|-r|--remove]]'
_help_ud = _usageTemplate + 'ud <expr>'
_slap_trigger = 'slap'
_featurerequest_trigger = 'featurerequest'


def processMessage(args, rawMessage, db_connection):
	if len(args) < 2:
		return None

	if args[0].lower() != '!' + _bottrigger.lower():
		return

	if args[1] == 'ping':
		return helper.botMessage('Hallu', _botname)

	elif args[1].lower() == 'ud':
		if len(args) <= 2:
			return helper.botMessage(_help_ud, _botname)
		term = stripFromBegin(rawMessage['message'], args[0:2])
		return helper.botMessage(ud.ud_parser(term), _botname)

	elif args[1].lower() == 'decide':
		if len(args) < 2:
			return
		antwort = '+' if ord(hashlib.sha1((rawMessage['message'].encode()+b'tpraR4gin8XHk_t3bGHZTJ206qc9vyV7LlUMTf655LNJDKGciVXKRLijqGkHgkpW <= Manfreds schlimmstes Geheimnis')).hexdigest()[0]) % 2 == 1 else '-'
		return helper.botMessage(antwort, _botname)

	elif args[1].lower() == 'sing':
		parser = argparse.ArgumentParser()
		parser.add_argument('song', nargs='?')
		parser.add_argument('-l', '--learn', action='store_true', dest='learn')
		parser.add_argument('-r', '--remove', action='store_true', dest='remove')
		try:
			parsedArgs = vars(parser.parse_args(args[2:]))
		except argparse.ArgumentError:
			return helper.botMessage(_help_sing, _botname)

		if ( parsedArgs['learn'] or parsedArgs['remove'] ) and parsedArgs['song'] is None:
			return helper.botMessage(_help_sing, _botname)

		if parsedArgs['learn']:
			return learntosing(parsedArgs['song'], db_connection)
		elif parsedArgs['remove']:
			return helper.botMessage(removeasong(parsedArgs['song'], db_connection), _botname)

		return helper.botMessage(singasong(db_connection.cursor()), _botname)

	elif args[1].lower() in [_slap_trigger, _featurerequest_trigger]:
		if len(args) == 2:
			return helper.botMessage('was meinst du?', _botname)

		target = stripFromBegin(rawMessage['message'], args[0:2])
		if args[1] == _slap_trigger:
			return helper.botMessage('%s schlägt %s'%(_botname, target), _botname)
		elif args[1] == _featurerequest_trigger:
			return helper.botMessage('Ich will %s'%target, _botname)

	else:
		return helper.botMessage(_help, _botname)



def learntosing(link, db_connection):
		cursor = db_connection.cursor()
		# ~ if e['user_id'] in self.singbans:
			# ~ return helper.botMessage(message = "Ich hab Niveau!", _botname)
		link = link.replace('http://','https://').strip()
		if link.startswith('youtube'):
			link = 'https://www.' + link
		elif link.startswith('www.'):
			link = 'https://'+s
		link = link.replace('youtube.de','youtube.com')
		p=re.compile('^https:\/\/www.youtube.com\/watch\?v=[a-zA-Z0-9\-_]{,20}$')
		if not p.match(link):
			return helper.botMessage("Die url passt nicht ganz.", _botname)
		query = cursor.execute(
					"SELECT id FROM songs WHERE link = ?;", (link,)
				).fetchone()
		if query is not None:
			return helper.botMessage("Das kenn ich schon.", _botname)
		# ~ vid = s.partition("?v=")[2]
		# ~ url = "https://www.googleapis.com/youtube/v3/videos?part=id&id=%s&key=%s"
		# ~ res = self.b.c.s.get(url%(vid,self.key))
		# ~ resjson = json.loads(res.text)
		# ~ if resjson['items'] == []:
			# ~ return dict(name = str(self), message = "Ich glaube das Video gibts nicht")
		cursor.execute("INSERT INTO songs (link) VALUES (?);", (link,))
		db_connection.commit()
		return helper.botMessage('Ich kann jetzt '+link+' singen.', _botname)


def songCount(cursor):
	return cursor.execute(
				'SELECT COUNT(*) FROM songs;'
			).fetchone()[0]


def singasong(cursor):
	anzahl = songCount(cursor)
	if anzahl is 0:
		return 'ICH KANN NICHT SINGEN!'
	songids = [x[0] for x in cursor.execute('SELECT id FROM songs;').fetchall()]
	return cursor.execute(
				'SELECT link FROM songs WHERE id = ?',
				(random.choice(songids),)
			).fetchone()[0]


def removeasong(link, db_connection):
	cursor = db_connection.cursor()
	songid = cursor.execute(
				'SELECT id FROM songs WHERE link == ?;', (link,)
			).fetchone()
	if songid is None:
		return helper.botMessage('Ich kenne '+link+' nicht.', _botname)
	cursor.execute('DELETE FROM songs WHERE id == ?;', (songid[0],))
	db_connection.commit()
	return helper.botMessage('Ich kann jetzt '+link+' nicht mehr singen.', _botname)
