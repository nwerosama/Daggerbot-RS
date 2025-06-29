#!/bin/bash

kubectl apply -f k-secret.yml && \
kubectl apply -f k.yml
