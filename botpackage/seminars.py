from botpackage.helper import helper

from requests import Session
from bs4 import BeautifulSoup
import pyparsing
import re

_bottrigger = "seminar"
_botname = "ein Orga"
_help = "Listet Informationen über die bevorstehenden Seminare."
_main_url = "https://qed-verein.de"

#BeautifulSoup Vorbereitung
_id = "node-%s"
_news_class = "node node-news-extern node-promoted node-sticky node-teaser"

def get_from_file(f, node):
    soup = BeautifulSoup(f, features='html.parser')
    return soup.find("div", {"id": _id%node}).getText()

def get_content(response, node):
    soup = BeautifulSoup(response.text, features='html.parser')
    return soup.find("div", {"id": _id%node}).getText()

def get_news(response):
    soup = BeautifulSoup(response.text, features='html.parser')
    ret = []
    for i in soup.findAll("tr"): #, {"class": _news_class}):
        title = i.find("h2")

        ret.append({"id": title.find("a")["href"].rpartition("/")[-1], "title": title.getText()})

    return ret

#Pyparsing Vorbereitung
_month_list = ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"]
_months = pyparsing.Or(pyparsing.Literal(i) for i in _month_list)

_day = pyparsing.Combine(pyparsing.Word(pyparsing.nums, max=2) + ".")
_month = pyparsing.Combine(_day | _months)
_year = pyparsing.Combine("20" + pyparsing.Word(pyparsing.nums, exact=2))
_weekday = pyparsing.Or(["Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag", "Sonntag"]) + pyparsing.Optional(",") + pyparsing.Optional(pyparsing.Literal("den") | "dem")

_date = pyparsing.Suppress(pyparsing.Optional(_weekday)) + _day.setResultsName("day") + pyparsing.Optional(_month.setResultsName("month") + pyparsing.Optional(_year.setResultsName("year")))

_noise = pyparsing.Suppress((~_date + pyparsing.Word(pyparsing.alphas + pyparsing.alphas8bit)) * (None, 4))

_full = _noise + _date

def get_dates(anchor, text):
    ret = None
    start = 0
    while start != -1:
        start = text.find(anchor, start + 1)
        try:
            return _full.parseString(text[start:])
        except:
            pass
    return None

def get_all():
    all_news = get_news(Session().get(_main_url))
    l = []
    for i in all_news:
        dates = {}
        for j in ["vo", "zu", "Anmeldeschluss"]:
            cont = get_content(Session().get("%s/node/%s"%(_main_url, i["id"])), i["id"])
            dates[j] = get_dates(j, cont)
        dates.update(i)
        l.append(dates)

    return l

_format_string = "{title}: {datestring}, Anmeldeschluss: {Anmeldeschluss[day]}{Anmeldeschluss[month]}{Anmeldeschluss[year]}"
def format_news(news):
    if news["Anmeldeschluss"] is None:
        news["Anmeldeschluss"] = "unbekannt"
    else:
        if news["Anmeldeschluss"]["month"] in _month_list:
            news[i]["Anmeldeschluss"] = "%02i."%_month_list.index(i)
            
    try:
        for i in ["month", "year"]:
            if i not in news["vo"]:
                news["vo"][i] = news["zu"][i]

        for i in ["vo", "zu"]:
            if news[i]["month"] in _month_list:
                news[i]["month"] = "%02i."%_month_list.index(i)

        datestring = "vom {vo[day]}{vo[month]}{vo[year]} bis zum {zu[day]}{zu[month]}{zu[year]}".format(**news)

    except Exception as e:
        datestring = "Datum unbekannt."
    
    return _format_string.format(**news, datestring = datestring)

def processMessage(args, rawMessage, db_connection):
    if len(args) == 1 and args[0].lower() == "!" + _bottrigger:
        news = get_all()
        return helper.botMessage("\n".join(map(format_news, news)), _botname)
