//could also use bit_set library, but fuck that.
use std::arch::asm;
use rand::Rng;

const N_PLAYERS: usize = 2;

const ALL: u64 = !0u64 >> (64-54);
const JOKERS: u64 = 3u64 << 52;
const SAME_NR: [u64; 54] = {
    let mut temp = [0u64; 54];
    let mut i: usize = 0;
    while i<13 { // Due to rust reasons I cannot use a for loop. Wtf. https://stackoverflow.com/questions/55479223/is-it-possible-to-write-a-const-function-that-folds-over-an-iterator
        let mut value: u64 = 1 << i;
        value |= 1 << i+13;
        value |= 1 << i+2*13;
        value |= 1 << i+3*13;
        temp[i] = value;
        temp[i+13] = value;
        temp[i+2*13] = value;
        temp[i+3*13] = value;
        i += 1;
    }
    //jokers
    temp[52] = JOKERS;
    temp[53] = JOKERS;
    temp
};
const SAME_CLASS: [u64; 54] = {
    let mut temp = [0u64; 54];
    let mut i: usize = 0;
    while i < 4 {
        let value: u64 = (1u64<<13-1) << i*13;
        let mut j: usize = 0;
        while j < 13 {
            temp[i*13+j] = value;
            j += 1;
        }
        i+=1;
    }
    temp[52] = JOKERS;
    temp[53] = JOKERS;
    temp
};
const CARDS_THAT_STACK: [u64; 54] = {
    let mut temp = SAME_NR;
    let mut i: usize = 0;
    while i < 54 {
        temp[i] |= SAME_CLASS[i];
        temp[i] |= JOKERS;
        i += 1;
    }
    temp[52] = ALL;
    temp[53] = ALL;
    temp
};

fn get_card_nr(card: u64) -> usize {
    #[cfg(target_arch = "x86_64")] {
        unsafe {
            let nr: usize;
            asm!(
                "bsr {0}, {1}",
                out(reg) nr,
                in(reg) card
            );
            nr
        }
    }
}
trait Player {
    fn chose(&self, own_cards: u64, playable: u64, last_card: u64);
}
struct BotPlayer {

}
impl Player for BotPlayer {
    fn chose(&self, own_cards: u64, playable: u64, last_card: u64) {

    }
}

fn print_binary(card_set: u64) {
    println!("{:0>64b}", card_set);
}

fn select_random_bit(v: u64, rng: &mut rand::rngs::ThreadRng) -> usize {
    //Source: https://graphics.stanford.edu/~seander/bithacks.html#SelectPosFromMSBRank
    //It doesn't exactly link you to the correct page hight, ctrl+f for "Select the bit position"

    let mut s: u64;        // Output: Resulting position of bit with rank r [1-64]
    let (a, b, c, d): (u64, u64, u64, u64); // Intermediate temporaries for bit count.
    let mut t: u64;          // Bit count temporary.

    // Do a normal parallel bit count for a 64-bit integer,                     
    // but store all intermediate steps.                                        
    a =  v - ((v >> 1) & (!0u64)/3);
    b = (a & (!0u64)/5) + ((a >> 2) & (!0u64)/5);
    c = (b + (b >> 4)) & (!0u64)/0x11;
    d = (c + (c >> 8)) & (!0u64)/0x101;
    t = (d >> 32) + (d >> 48);
    // Now do branchless select!                                                
    let count: u64 = ((((b + (b >> 4)) as u128 & 0xF0F0F0F0F0F0F0Fu128) * 0x101010101010101u128) & ((1<<64)-1)) as u64 >> 56;
    let mut r: u64 = 2;//1+rng.gen_range(0..count); //The input value; the rank of the bit to select.
    s  = 64;
    s -= ((t - r) & 256) >> 3; r -= t & ((t - r) >> 8);
    t  = (d >> (s - 16)) & 0xff;
    s -= ((t - r) & 256) >> 4; r -= t & ((t - r) >> 8);
    t  = (c >> (s - 8)) & 0xf;
    s -= ((t - r) & 256) >> 5; r -= t & ((t - r) >> 8);
    t  = (b >> (s - 4)) & 0x7;
    s -= ((t - r) & 256) >> 6; r -= t & ((t - r) >> 8);
    t  = (a >> (s - 2)) & 0x3;
    s -= ((t - r) & 256) >> 7; r -= t & ((t - r) >> 8);
    t  = (v >> (s - 1)) & 0x1;
    s -= ((t - r) & 256) >> 8;
    s = 65 - s;

    (s-1) as usize
}
fn select_random(from_stack: &mut u64, to_stack: &mut u64, n: usize, rng: &mut rand::rngs::ThreadRng) {
    for _ in 0..n {
        let random_bit_position = select_random_bit(*from_stack, rng);
        let selected_card = 1 << random_bit_position;
        *from_stack ^= selected_card;
        *to_stack |= selected_card;
    }
}

fn play_game(players: &Vec<&dyn Player>, verbose: bool, randomStart: bool) -> usize {
    let mut stack: u64 = ALL;
    let mut in_hands: [u64; N_PLAYERS] = [0u64; N_PLAYERS];
    
    let mut rng = rand::thread_rng();
    for player in 0..N_PLAYERS {
        select_random(&mut stack, &mut in_hands[player], 7, &mut rng);
    }
    let mut lay_stack = 0u64;
    let mut last_card = 0u64;
    select_random(&mut stack, &mut last_card, 1, &mut rng);
    lay_stack |= last_card;

    let mut player_turn: usize = 0;
    loop {
        player_turn %= players.len();
        let p: &dyn Player = players[player_turn];

        let playable: u64 = in_hands[player_turn] & CARDS_THAT_STACK[get_card_nr(last_card)];
        if playable == 0 {
            //draw a card.
            continue;
        }

        if in_hands[player_turn] == 0 {
            return player_turn;
        }
        player_turn += 1;
    }
}

fn main() {
    println!("Welcome to the pesten simulator!");
    let a: BotPlayer = BotPlayer{};
    let b: BotPlayer = BotPlayer{};
    let players: Vec<&dyn Player> = vec![&a, &b];
    let winner = play_game(&players, true, false);
    println!("{}", winner);
}
