# VHS script for docs/assets/demo.gif; run it through scripts/record-demo.sh,
# which builds Locmetry and stages the pinned Immich showcase it measures.
Set Shell "bash"
Set FontSize 13
Set Width 1120
Set Height 960
Set Padding 20
Set TypingSpeed 45ms
Set Framerate 24

Hide
Type "cd /tmp/locmetry-demo/immich && clear"
Enter
Show

Type "locmetry"
Sleep 400ms
Enter
Sleep 6s

Hide
Type "clear"
Enter
Show
Type "locmetry explain packages/sdk/src/fetch-client.ts"
Sleep 400ms
Enter
Sleep 5s

Hide
Type "clear"
Enter
Show
Type "locmetry compare ../v3.2.0.json ../v3.3.4.json --tables-only"
Sleep 400ms
Enter
Sleep 8s
