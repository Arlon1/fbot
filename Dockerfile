FROM python:3
COPY entrypoint.sh /
ADD fbot.py /
ADD botpackage /botpackage
RUN pip install requests websocket-client beautifulsoup4 pyparsing parsedatetime
ENTRYPOINT ["/entrypoint.sh"]
