import numpy as np

from botpackage.helper import helper
from botpackage.helper.mystrip import _space_chars
import botpackage.fantasynames.fishNames as fishNames
import botpackage.fantasynames.lovecraftianNames as lovecraftianNames


_interact_chance = 1/7000
_cthulhu_chance = 1./300

_reactions = ["*bites {victim} for {dmg} dmg*", "blub. blub. blub.", "blub! *plätscher*", "*blub! *glitzer*", "*lässt sich von {victim} kraulen*", "Ladet die QED-App herunter, denn sie ist toll. (https://github.com/jbb01/QED/)"]

_victims = {}


def addVictim(v):
    global _victims
    P = np.random.random(len(_reactions))
    P = np.cumsum(P/np.sum(P))

    if np.random.random() <= _cthulhu_chance:
         name = lovecraftianNames.randomName()
    else:
         name = fishNames.randomName()

    _victims[v] = {
        "P": P,
        "carp_name": name
    }
    return

def interact(victim):
    global _victims
    v = victim.lower()

    if not v in _victims:
        addVictim(v)

    ran = np.random.random()
    index = np.argmax(_victims[v]["P"] >= ran)
    msg = _reactions[index]
    name = _victims[v]["carp_name"]
    # potentially useful variables
    dmg = np.random.randint(1,5)
    return helper.botMessage(msg.format(**globals(), **locals()), name)

def processMessage(args, rawMessage, db_connection):
    user = rawMessage["name"].strip(_space_chars)

    if np.random.random() <= _interact_chance:
        return interact(user)
