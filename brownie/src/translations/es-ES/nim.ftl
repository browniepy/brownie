nim = nim
    .description = Juego de nim tipo cero
    .user = usuario
    .user-description = Usuario rival de la apuesta
    .bios = bios
    .bios-description = Cantidad de bios para apostar

nim-request =
    Apuesta de nim tipo cero para { $user }
    En juego { $amount } bios

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
    { $loser } perdió la apuesta de nim
    El ganador es { $winner }
