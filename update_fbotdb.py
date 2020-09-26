#!/usr/bin/env python
import argparse
import re

from bs4 import BeautifulSoup
import sqlite3

if __name__ == '__main__':

    # -----
    # argument parsing
    # -----
    parser = argparse.ArgumentParser()
    parser.add_argument('html_file')
    parser.add_argument('sqlite_db')

    args = vars(parser.parse_args())

    # -----
    # html parsing
    # -----

    html_raw = open(args['html_file'])

    html_tree = BeautifulSoup(html_raw, features='lxml')

    html_table = html_tree.find(id="personen")
    html_tbody = html_table.find('tbody')
    bs_list = list(html_tbody.find_all(name='tr', attrs={"class": "data"}))

    # our user database
    data = []

    for person in bs_list:
        data.append(
            {
                'name': person.find(
                    attrs={"class": "personennachname"}
                    ).text.replace('\xa0', ''),
                'vorname': person.find(
                    attrs={"class": "personenvorname"}
                    ).text.replace('\xa0', ''),
                'userid': int(
                    person.find(
                        name='td',
                        attrs={'id': re.compile('^personen_')}
                    ).get('id').replace('personen_', '')
                    )
            }
        )

    # -----
    # reading and querying the sql database
    # -----

    conn = sqlite3.connect(args['sqlite_db'])
    c = conn.cursor()

    db_list = c.execute("SELECT userid FROM nicknames").fetchall()
    db_list = set(
        [x[0] for x in db_list]
        )

    # setminus
    diff = list(
        set(
            [x['userid'] for x in data]
            ).difference(db_list)
        )

    # diff UNION data
    toAdd = [
        [x['vorname']+x['name'], x['userid']]
        for x in data
        if x['userid'] in diff
        ]

    # -----
    # database writing
    # -----
    c.executemany('INSERT INTO nicknames'+\
                  '(nickname, userid, deletable)'+\
                  'VALUES (?, ?, 1);',
                  toAdd
                  )
    conn.commit()
    conn.close()
