# command metadata

work = work
    .description = work to earn money

balance = balance
    .description = view a balance
    .user = user
    .user-description = user to view their balance

dth = dth
    .description = deja caer el pañuelo de usogui
    .user = user
    .user-description = usuario para desafiar

nim = nim
    .description = nim type zero from kakegurui
    .user = user
    .user-description = user to challenge
    .amount = amount
    .amount-description = amount of money to bet

# profile related

member-role = Member
referee-role = Referee
leader-role = Leader

profile = { $name } ({ $role })

kariume-work =
    You sold kariume and got { $amount }.

balance-self = You have { $balance } bills.

balance-other = { $name } has { $balance } bills.

proposal = { $name }, do you want to accept the bet?

wrong-inter = This button isn't for you.

dh-gamble-proposal =
    Drop the Handkerchief game.
    { $user }, do you accept the bet?

# messages drop the handkerchief

dh-start =
    Bet accepted.
    First round starts at { $time }.

dh-inround =
    Round started.
    { $checker } checks and { $dropper } drops.

dh-round-fail-alive =
    { $checker } was revived.
    Preparing next round...

dh-round-fail-death =
    Bet finished.
    { $checker } couldn't be revived.

dh-try-reanimate =
    { $checker } failed to check.
    Drug injected, attempting to revive...

dh-round-end-cok =
    { $checker } successfully checked.
    { $seconds } seconds added.

dh-round-expired =
    Bet cancelled due to lack of participation.

dh-stats =
    { $checker }, wasted time { $checkerWasted }s;
    Near-death time { $checkerDeath }.

    { $dropper }, wasted time { $dropperWasted }s;
    Near-death time { $dropperDeath }.

already-dropped =
    You already dropped the handkerchief.

#buttons drop the handkerchief

dh-drop-btn = Drop
dh-check-btn = Check

# buttons general

accept-btn = Accept
decline-btn = Decline

replay-btn = Play Again

# buttons blackjack

hit-btn = Hit
hold-btn = Hold
double-btn = Double
stats-btn = Stats
