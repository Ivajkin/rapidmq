# RapidMQ API Documentation

## Table of Contents
1. [Introduction](#introduction)
2. [Producer API](#producer-api)
   - [Initialization](#producer-initialization)
   - [Sending Messages](#sending-messages)
   - [Error Handling](#producer-error-handling)
3. [Consumer API](#consumer-api)
   - [Initialization](#consumer-initialization)
   - [Receiving Messages](#receiving-messages)
   - [Message Acknowledgment](#message-acknowledgment)
   - [Error Handling](#consumer-error-handling)
4. [Common Data Structures](#common-data-structures)
5. [Error Codes](#error-codes)

## Introduction

The RapidMQ API provides two main interfaces for interacting with the message queue system:
- Producer API: For sending messages to topics
- Consumer API: For receiving messages from topics

This documentation covers the core functionality of both APIs. Code examples are provided in Python, but similar patterns apply to other supported languages.

## Producer API

### Producer Initialization