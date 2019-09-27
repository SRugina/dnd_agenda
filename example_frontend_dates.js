let date1 = {
    "id": 1,
    "slug": "epic-session",
    "title": "Epic Session",
    "description": "stuff and things",
    "dm": 1,
    "session_date": "2019-09-21T15:30:00.000Z",
    "colour": "red"
};

let date2 = {
    "id": 2,
    "slug": "epic-session-2",
    "title": "Epic Session 2",
    "description": "stuff and things",
    "dm": 1,
    "session_date": "2019-10-21T11:30:00.000Z",
    "colour": "red"
};

let date3 = {
    "id": 3,
    "slug": "epic-session-3",
    "title": "Epic Session 3",
    "description": "stuff and things",
    "dm": 1,
    "session_date": "2019-09-30T17:30:00.000Z",
    "colour": "red"
};

let arr = [date1, date2, date3];

let new_arr = [];

for (date of arr) {
    new_arr.push({
        dot: date.colour,
        key: date.slug,
        //highlight: true,
        dates: new Date(date.session_date),
        popover: {
            label: date.title,
        },
    })
}

console.log(new_arr);