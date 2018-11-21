# -*- coding: utf-8 -*-
from botpackage.helper import helper
from datetime import datetime, timedelta
from random import random

praiseChance = 0.33
waittime = timedelta(minutes=60)
    
#praise them \o/
targets = {
    "Jana": {
        "origin": "	 	  	    			  	    	 Jörn",
        "lastTime": None
    },
    "QED": {
        "origin": "Jörn",
        "lastTime": None
    }
}

def norm(name):
    return name.lower().strip()

def processMessage(args, rawMessage, db_connection):

    targetOptions = [t for t in targets if norm(t) == norm(rawMessage["name"])]
    if not targetOptions:
        return None #no target, no praise
    
    target = targetOptions[0] #unpack
    origin = targets[target]["origin"]
    lastTime = targets[target]["lastTime"]
    
    if lastTime and waittime > datetime.now()-lastTime: 
        return None #too often praise is not good
    
    if random() > praiseChance:
        return None #too many praise is not good
    
    targets[target]["lastTime"] = lastTime = datetime.now()
    return helper.botMessage("praise the %s \o/" % target, origin) #praise the fbot \o/

