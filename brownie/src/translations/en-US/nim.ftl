nim = nim
    .description = Juego de nim tipo cero
    .user = user
    .user-description = Usuario rival de la apuesta
    .bios = bios
    .bios-description = Cantidad de bios para apostar

nim-request =
    Juego de nim tipo cero para { $user }
    Apostando { $amount } bios

nim-choose-card =
    Elige una de tus cartas disponibles

nim-start =
    No hay carta anterior tirada
    { $user } inicia eligiendo una carta

nim-new-game =
    No hay carta anterior tirada
    { $user } inicia eligiendo una carta

nim-round-info =
    Anterior carta tirada { $card }
    { $userA } es tu turno de elegir

nim-round-lose =
    { $loser } perdió la ronda de nim
    Punto para { $winner }

nim-end =
    { $loser } perdió el juego de nim
    El ganador es { $winner }
