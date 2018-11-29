# -*- coding: utf-8 -*-
from botpackage.helper import helper
import botpackage.fantasynames.fishNames as fishNames
import botpackage.fantasynames.lovecraftianNames as lovecraftianNames
import numpy as np

_interact_chance = 1./700
_cthulhu_chance = 1./30

_reactions = ["*bites {victim} for {dmg} dmg*", "blub. blub. blub.", "blub! *plätscher*", "*blub! *glitzer*", "*lässt sich von {victim} kraulen*"]

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
    #potentially useful variables
    dmg = np.random.randint(1,5)
    return helper.botMessage(msg.format(**globals(), **locals()), name)

def processMessage(args, rawMessage, db_connection):
    user = rawMessage["name"].strip()

    if np.random.random() <= _interact_chance:
        return interact(user)
