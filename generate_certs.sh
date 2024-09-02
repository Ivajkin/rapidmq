#!/bin/bash

# Generate CA key and certificate
openssl req -x509 -newkey rsa:4096 -days 365 -nodes -keyout ca-key.pem -out ca-cert.pem -subj "/C=US/ST=California/L=San Francisco/O=RapidMQ/OU=Dev/CN=RapidMQ CA"

# Generate server key and certificate signing request (CSR)
openssl req -newkey rsa:4096 -nodes -keyout key.pem -out server.csr -subj "/C=US/ST=California/L=San Francisco/O=RapidMQ/OU=Dev/CN=localhost"

# Sign the server CSR with the CA
openssl x509 -req -in server.csr -CA ca-cert.pem -CAkey ca-key.pem -CAcreateserial -out cert.pem -days 365

# Clean up
rm server.csr ca-cert.srl