#!/bin/bash
./target/debug/volley &
PID=$!
sleep 3
kill $PID 2>/dev/null