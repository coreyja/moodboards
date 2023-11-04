# Tasks

- [ ] Deploy this app somewhere
- [ ] Make sure Sqlite DB is persisted across deploys
- [ ] Create Better Layout of picture with rating buttons
- [ ] Add LiteFS to replicate the DB across multiple containers

## Done on Second Stream

- [x] Create a Sqlite Database
- [x] Clicking either rating button will store the rating for the picture

Today we worked on getting a Sqlite database setup and working with the
saving PictureRatings to the DB.

Now you can upvote or downvote any image, and it will be saved to the DB.

We are currently just creating a new MoodBoard row each time the server boots.
This is help in AppState at the moment, but won't be long term

We created a very basic Data Diagram at: <https://link.excalidraw.com/readonly/NUDxops7knZMQLFkgFmO>

We don't have any User stuff modeled yet, but we will get there.

## Done on First Stream

- [x] Create Basic Layout of picture with rating buttons
- [x] Clicking either rating button will change the picture

We got the WASM setup all taked care of here! We are ready to start building the app!

We got a very basic attribute working for clicking on a button and replacing a specified id
with the contents of a GET request.
