# habit-tracker
Minimalist command-line habit tracker, written in Rust.

Everything is WIP, but here is a quick list of working functions you can call straight from the command line.

Habit Tracker Commands: 

help -- prints out this list inside your terminal

skip <habit> <opt date> -- mark a habit as skipped, defaults to today

complete <habit> <opt date> -- mark a habit as complete, defaults to today

fail <habit> <opt date> -- mark a habit as failed, defaults to today

increment <habit> <value> <opt date> -- add value to a habit with a numerical goal, defaults to today)

set <habit> <value> <opt date> -- overwrites existing value for a habit, defaults to today

reset <habit> <opt date> -- reset a habit node, defaults to today

add_habit <habit name> <desc> <goal> <opt enabled days as 1-3-5-7 etc> -- adds a new habit to track

remove_habit <habit name> -- deletes a habit and all of that habit's history

hide_habit <habit name> -- stops showing a habit, but keeps the history saved and will not mark days as skipped

list <opt date> -- shows a status list of all active habits at the specified date, defaults to today

history <habit> -- shows to-date data of the specified habit, tracking % of completed days




User data is stored locally for now, inside of the root folder as "userdata.bin".  If you need to reset your data entirely, you can delete that file OR call reset_all (warning: currently unguarded! this will delete your data instantly without confirmation!)
