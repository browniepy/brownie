nim = nim
    .description = nim tipo cero
    .user = usuario
    .user-description = usuario rival, puedes jugar contra la ia
    .bios = bios
    .bios-description = apuesta de bios

nim-ur-cards =
    estas son tus cartas, ¡elige bien!

nim-start =
    { $user } inicia eligiendo una carta

table-sum = la mesa ahora suma

nim-round-info =
    { $user } ha jugado la carta { $card }
    { $total ->
        [0] ¡la mesa sigue valiendo cero!
        [1] { table-sum } { $total }
        [2] { table-sum } { $total }
        [3] { table-sum } { $total }
        [4] { table-sum } { $total }
        [5] { table-sum } { $total }
        [6] { table-sum } { $total }
        [7] { table-sum } { $total }
        [8] { table-sum } { $total }
        [9] ¡{ table-sum } { $total }!
       *[other] el valor final de la mesa es { $total }
    }

nim-turn =
    turno de elegir para { $user }

nim-round-lose =
    { $user } perdió esta ronda
    la carta { $card } sobrepasó el límite

nim-game-lose =
    { $loser } perdió contra { $winner }
    la carta { $card } sobrepasó el límite
