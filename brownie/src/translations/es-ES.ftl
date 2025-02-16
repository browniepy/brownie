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

contradiction = contradiction
    .description = juego de contradicción
    .user = usuario
    .user-description = usuario rival de la apuesta
    .amount = cantidad
    .amount-description = cantidad de la apuesta

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

# Contradiction

fround = 1era
sround = 2da
tround = 3era

fgame = 1er
sgame = 2do
tgame = 3er

iron = Hierro
wood = Madera
rubber = Caucho


contradict-open =
    Apuesta Kakerou abierta


gun-iron =
    { $defender } desvió el disparo de { $attacker }
    gracias al Escudo de { iron }

gun-wood-rubber =
    { $defender } no pudo detener el disparo
    por usar el Escudo de { $material }

katana-wood =
    { $defender } fue cortado por Katana
    por usar el Escudo de { wood }

katana-rubber =
    { $defender } fue cortado levemente por Katana
    usando el Escudo de { rubber }

katana-iron =
    { $defender } detuvo el ataque de Katana
    con el Escudo de { iron }

taser-iron =
    { $defender } recibió una Descarga fuerte
    por usar el Escudo de { iron }

taser-wood =
    { $defender } apenas sintió la Descarga
    gracias al Escudo de { wood }

taser-rubber =
    { $defender } bloqueó la Descarga
    gracias al Escudo de { rubber }


first-round-info =
    { $shields } elige los Escudos
    y { $weapons } las Espadas


choose-phase =
    Hora de elegir los Escudos y Espadas

bet-phase =
    Objetos elegidos
    Tiempo para hacer sus apuestas de bios

bet-info =
    Las apuestas fueron
    { $a } { $a-bios } bios, { $b } { $b-bios } bios


contradict-end =
    { $loser } acabó saliéndose de la línea
    el ganador es { $winner }


bet-again =
    Por favor, apuesten sus bios otra vez
    no pueden apostar lo mismo

invalid-bet =
    Esa cantidad de bios no es válida
