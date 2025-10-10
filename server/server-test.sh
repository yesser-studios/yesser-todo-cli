#!/bin/bash

curl http://127.0.0.1:6982/add -X POST --json '"hello"'
curl http://127.0.0.1:6982/tasks
curl http://127.0.0.1:6982/remove -X DELETE
