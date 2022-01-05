FROM python:3 AS builder
WORKDIR /
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

ARG PROJECT='paper'
ARG VERSION='1.18.1'

COPY fetch-paper-api.py .
RUN python fetch-paper-api.py -p $PROJECT -v $VERSION
