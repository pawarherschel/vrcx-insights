# VRCX Insights

this is a simple program to find out the various friend circles which might exist via the data collected by vrcx

## Usage

1. make a new folder called `db` in the same directory as the executable
2. copy the `VRCX.sqlite3` from `%APPDATA%\VRCX` into the `db` folder
3. copy your user id

   you can find our your user id by
    1. opening vrcx
    2. click on your name on right panel
    3. at the bottom of the info card, there should be `User ID`
    4. your user id should look similar to `usr_aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee`
4. make a new file called `owner_id.txt` in the same directory as the executable
5. paste the user id into the file
6. check `sorted_undirected_graph.ron` to see the results