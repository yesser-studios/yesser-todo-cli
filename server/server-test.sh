#!/bin/bash

curl http://127.0.0.1:6982/add -X POST --json '"hello"'
curl http://127.0.0.1:6982/tasks
curl http://127.0.0.1:6982/done -X POST --json '0'
curl http://127.0.0.1:6982/undone -X POST --json '0'
curl http://127.0.0.1:6982/remove -X DELETE --json '0'
curl http://127.0.0.1:6982/add -X POST --json '"hello-clear"'
curl http://127.0.0.1:6982/clear -X DELETE
curl http://127.0.0.1:6982/tasks
curl http://127.0.0.1:6982/add -X POST --json '"hello-clear-done"'
curl http://127.0.0.1:6982/done -X POST --json '0'
curl http://127.0.0.1:6982/cleardone -X DELETE
curl http://127.0.0.1:6982/tasks