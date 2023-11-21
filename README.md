# VRCX Insights

this is a simple program to find out the various friend circles which might exist via the data collected by vrcx

keep in mind that the program tries to use ALL your threads, so if you have a cpu with low number of threads then it
will take a while to run. the amount of time it requires to run is also dependent on how big the database table is.

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
6. run the executable
7. check `sorted_undirected_graph.ron` to see the results

## What the results mean

### sorted_undirected_graph.ron

this file contains a list of all the friend circles sorted by size. the first entry is the largest friend circle, the
second entry is the second-largest friend circle, and so on.

### graph2_sorted.ron

this is the same as `sorted_undirected_graph.ron` except it is in directed graph. this means that if `A` is friends with
`B` then `B` is not necessarily friends with `A`.

the information is in form

```rust
/// The schema is a vector of (name, HashMap<name, (count, percentage, percentile-ish)>)
/// - count: number of times the name was seen
/// - percentage: percentage of the total number of samples rounded to 2 decimal places
///          (i.e. count / total_samples * 100)
/// - percentile-ish: its similar to percentile but not quite. It's the percentage of the
///         count from the highest count + 1 rounded to 2 decimal places
///        (i.e. count / (highest_count + 1) * 100)
///
/// The first `String` is user `A`
/// The second `String` is user `B`
type Schema = Vec<(String, HashMap<String, (u32, f64, f64)>)>;
```

example

```
[
    ("A", {
        "B": (275, 2.23, 93.22),
        "C": (168, 1.36, 56.95),
        "D": (160, 1.3, 54.24),
        "E": (264, 2.14, 89.49),
        "F": (210, 1.71, 71.19),
        "G": (294, 2.39, 99.66),
        "H": (201, 1.63, 68.14),
    }),
    ("B", {
        "A": (273, 16.89, 99.64),
        "I": (87, 5.38, 31.75),
        "J": (102, 6.31, 37.23),
    }),
    ("G", {
        "K": (27, 5.05, 9.25),
        "A": (291, 54.39, 99.66),
        "D": (61, 11.4, 20.89),
    }),
]
```

### graph.ron

this is the first graph generated from the database

the information is in form

```rust
/// The schema is a map of Users to maps of other users and the number of times they appeared
type Schema = HashMap<String, HashMap<String, u32>>;
```

## How does it work?

i just made up the heuristic that if percentage > 0.05 or percentile > 0.5 then theyre friends lol

(someone please make the heuristic better)
