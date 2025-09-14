# Server part

This code runs on the server which handles authentication of the user, and acts as a backend relay for the mal-cli, keeping the client secret hidden.

The server is there just so the user doesn't have to create their own client id and client secret to log themselves in via my anime list credentials. 
this code is running on an external server hosted by me. 
if you prefer to have this running on you own server or in your own local machine, you can do that aswell, but this will require you to create the client id and client secret yourself.
here is a step by step guide on how to host the server part on your own machine:

### step 1:
head to https://myanimelist.net/apiconfig and create the client id and secret:
