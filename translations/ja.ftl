# command metadata

work = work
    .description = work to earn money

balance = balance
    .description = view a balance
    .user = user
    .user-description = user to view their balance

dth = dth
    .description = deja caer el pañuelo de usogui
    .user = usuario
    .user-description = usuario para desafiar

nim = nim
    .description = nim type zero from kakegurui
    .user = user
    .user-description = user to challenge
    .amount = amount
    .amount-description = amount of money to bet

# profile related

member-role = メンバー
referee-role = 審判
leader-role = リーダー

profile = { $name } ({ $role })

kariume-work =
    カリュームを売却し、{ $amount } を獲得しました。

balance-self = 所持金は { $balance } 円です。

balance-other = { $name } の所持金は { $balance } 円です。

proposal = { $name } さん、賭けを受け入れますか？

wrong-inter = このボタンはあなたのためではありません。

dh-gamble-proposal =
    ハンカチ落としゲームです。
    { $user } さん、賭けを受け入れますか？

# messages drop the handkerchief

dh-start =
    賭けが成立しました。
    第一ラウンドは { $time } に開始します。

dh-inround =
    ラウンド開始。
    { $checker } が確認し、{ $dropper } が落とします。

dh-round-fail-alive =
    { $checker } は蘇生しました。
    次のラウンドを準備しています…

dh-round-fail-death =
    賭けは終了しました。
    { $checker } は蘇生できませんでした。

dh-try-reanimate =
    { $checker } は確認に失敗しました。
    薬物を注入し、蘇生を試みています…

dh-round-end-cok =
    { $checker } は確認に成功しました。
    { $seconds } 秒が追加されました。

dh-round-expired =
    参加不足のため賭けはキャンセルされました。

dh-stats =
    { $checker }、無駄にした時間 { $checkerWasted }秒;
    仮死時間 { $checkerDeath }秒。

    { $dropper }、無駄にした時間 { $dropperWasted }秒;
    仮死時間 { $dropperDeath }秒。

already-dropped =
    既にハンカチを落としました。

#buttons drop the handkerchief

dh-drop-btn = 落とす
dh-check-btn = 確認する

# buttons general

accept-btn = 受諾
decline-btn = 拒否

replay-btn = もう一度プレイ

# buttons blackjack

hit-btn = ヒット
hold-btn = ホールド
double-btn = ダブル
stats-btn = 統計
