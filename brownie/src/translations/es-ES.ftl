# command metadata

work = trabajar
    .description = trabaja para conseguir dinero

balance = balance
    .description = mira un balance
    .user = usuario
    .user-description = usuario para ver su balance

dth = dth
    .description = deja caer el pañuelo de usogui
    .user = usuario
    .user-description = usuario para desafiar

nim = nim
    .description = nim tipo cero de kakegurui
    .user = usuario
    .user-description = usuario para desafiar
    .amount = cantidad
    .amount-description = cantidad de dinero para apostar

oldmaid = oldmaid
    .description = juego de oldmaid
    .user = usuario
    .user-description = usuario para desafiar
    .amount = cantidad
    .amount-description = cantidad de dinero para apostar

# command messages

balance-self =
    Tienes { $balance } balance

balance-other =
    { $name } tiene { $balance } balance

proposal =
    Toca el botón para unirte

proposal-decline =
    Apuesta rechazada

wrong-inter =
    Este botón no es para ti.

# profile related

member-role = Miembro
referee-role = Referí n{ $range }
leader-role = Líder

profile = { $name } ({ $role })

kariume-work =
    Vendiste kariume y conseguiste { $amount }.

dh-gamble-proposal =
    Juego del pañuelo.
    { $user }, ¿aceptas la apuesta?


# messages drop the handkerchief

dh-start =
    Apuesta aceptada.
    Primera ronda inicia a las { $time }.

dh-inround =
    Ronda iniciada.
    { $checker } comprueba y { $dropper } tira.

dh-round-fail-alive =
    { $checker } fue reanimado.
    Preparando la siguiente ronda...

dh-round-fail-death =
    Apuesta finalizada.
    { $checker } no pudo ser reanimado.

dh-try-reanimate =
    { $checker } falló al comprobar.
    Droga inyectada, intentando reanimar...

dh-round-end-cok =
    { $checker } comprobó con éxito.
    Se le añadieron { $seconds } segundos.

dh-round-expired =
    Apuesta cancelada por falta de participación.

dh-stats =
    { $checker }, tiempo desperdiciado { $checkerWasted }s;
    Tiempo de casi muerte { $checkerDeath }.

    { $dropper }, tiempo desperdiciado { $dropperWasted }s;
    Tiempo de casi muerte { $dropperDeath }.

already-dropped =
    Ya tiraste el pañuelo.

#buttons drop the handkerchief

dh-drop-btn = Soltar
dh-check-btn = comprobar

# buttons general

accept-btn = Aceptar
decline-btn = Rechazar

replay-btn = Jugar otra vez

# buttons blackjack

hit-btn = Tomar
hold-btn = Mantener
double-btn = Doblar
stats-btn = Estadísticas

choose-card-btn = Elegir carta

zero = Zero
one = One
two = Two
three = Tree

# Nim type zero

ntz-proposal =
    Nueva apuesta para { $user }

ntz-round-first-state =
    { $user } tiene el primer turno
    La mesa está vacía

ntz-round-state =
    { $userA } puso un { $card }
    Turno de { $userB }

ntz-round-set =
    { $user } perdió esta ronda
    La mesa final es { $table }

ntz-game-set =
    { $userA } perdió el juego
    La mesa final es { $table }

ntz-choose-card =
    Elige una de tus cartas

# Oldmaid

om-proposal =
    Nueva apuesta para { $user }

om-first-turn =
    { $userJoker } tiene el Joker
    { $userA }, empieza eligiendo una carta

om-turn =
    { $userJoker } tiene el Joker
    { $userA }, elige una carta de { $userB }

om-choosed-card =
    { $user } tomó la carta { $card }
    Pares descartados { $pairs }

om-end =
    { $winner } tomó la carta { $card }
    { $loser } pierde por tener el Joker

om-not-your-turn =
    Espera tu turno para poder elegir

# Bets

with-amount =
    La apuesta es de { $amount }

free-bet =
    No hay nada en juego

membresy-bet =
    Se está apostando una membresía

# Jobs

work-none = Te encontraste ${ $amount } en el suelo
work-referee = Ganaste ${ $amount } por presidir una apuesta
work-waiter = Serviste en un maid café y ganaste { $amount }
work-concierge = Limpiaste una habitación y ganaste { $amount }
work-fighter = Ganaste ${ $amount } en un torneo de lucha
