import sqlite3
from botpackage.helper import helper

from requests import Session
import re
from bs4 import BeautifulSoup

_bottrigger = "qedpoll"
_botname = 'Nicholas'
_help = "!qedpoll [<pollnode> | <pollname> | <pollnode> <pollname>]"
_url = "https://qed-verein.de/node/%s"
_regex = re.compile("(\d{1,3})% \((\d+) Stimmen*\)")

fourths = ["", chr(0x258e), chr(0x258c), chr(0x258a), chr(0x2588)]

def processMessage(args, rawMessage, db_connection):
    parsedArgs = list(filter(lambda x: len(x) > 0, args))

    if len(parsedArgs) < 1 or parsedArgs[0].lower() != "!" + _bottrigger:
        return

    cursor = db_connection.cursor()

    if len(parsedArgs) == 2:
        return get_poll(parsedArgs[1], db_connection)
    elif len(parsedArgs) == 3:
        return add_pollname(parsedArgs[1], parsedArgs[2], db_connection)
    else:
        return helper.botMessage(_help, _botname)

def add_pollname(num, name, db_connection):
    if not num.isnumeric():
        return helper.botMessage("%s ist keine valide Pollnummer."%(num), _botname)
    if name[0].isnumeric():
        return helper.botMessage("Der Pollname darf nicht mit einer Zahl beginnen.", _botname)

    cursor = db_connection.cursor()
    dbnum = get_num_from_name(name, cursor)

    if dbnum is not None:
        return helper.botMessage("Der Pollname %s ist schon für %s vergeben."%(name, dbnum[0]), _botname)

    cursor.execute("INSERT INTO qedpoll(name, num) values(?,?)", (name, num))
    db_connection.commit()

    return helper.botMessage("Die Poll %s hat nun den Namen %s."%(num, name), _botname)


def get_poll(poll, db_connection):
    if poll[0].isnumeric():
        return get_poll_from_num(poll)
    else:
        return get_poll_from_name(poll, db_connection.cursor())

def get_num_from_name(poll, cursor):
    cursor.execute("SELECT num FROM qedpoll WHERE lower(name) == ?;", (poll.strip().lower(),))
    return cursor.fetchone()

def get_poll_from_name(poll, cursor):
    match = get_num_from_name(poll, cursor)
    if match is None:
        return helper.botMessage("Ich kenne Pollname %s nicht."%poll, _botname)
    else:
        return get_poll_from_num(match[0])

def get_poll_from_num(poll):

    if not poll.isnumeric():
        return helper.botMessage("%s ist keine valide Pollnummer"%poll, _botname)
    res = Session().get(_url%poll)
    if not res.ok:
        return helper.botMessage("Ich kenne die Pollnummer %s nicht."%poll, _botname)

    soup = BeautifulSoup(res.text, features='html.parser')

    title = soup.find("h1", {"class": "with-tabs"}).getText()

    answers = [title,"\n"]
    polldiv = soup.find("div", {"class": "poll"})
    if polldiv == None:
        return helper.botMessage("Die Nummer %s gehört zu keiner Umfrage."%poll, _botname)

    for i in polldiv.findAll("div", {"class": "text"}):
        opinion = i.getText().strip()
        votes = i.findNextSibling("div", {"class": "percent"}).getText().strip()

        answers.append(format_vote(opinion, votes))

    answers.append(polldiv.find("div", {"class": "total"}).getText())



    return helper.botMessage('\n'.join(answers), _botname)

def format_vote(opinion, votes):
    m = _regex.match(votes)
    if not m:
        return "Invalid formated votes %s for opinion %s."%(votes, opinion)

    percentage, voters = map(int, m.groups())

    bar = ''.join((fourths[4] for i in range(percentage//4))) + fourths[percentage%4]

    return "%s : %s Stimmen (%s%%)\n%s"%(opinion, voters, percentage, bar)
