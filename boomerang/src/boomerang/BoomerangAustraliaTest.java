package boomerang;

import static org.junit.Assert.*;
import org.junit.Before;
import org.junit.Test;

import boomerang.BoomerangAustralia.Card;
import boomerang.BoomerangAustralia.Player;

import java.io.ByteArrayOutputStream;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;
import java.io.PrintStream;
import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.net.Socket;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Scanner;

public class BoomerangAustraliaTest {
    // The game content
    private BoomerangAustralia game;
    // Capture stdout
    private ByteArrayOutputStream outContent;

    public class DummyPlayer {

        public void client(String ipAddress) throws Exception {
            // Connect to server
            Socket aSocket = new Socket(ipAddress, 2048);
            ObjectOutputStream outToServer = new ObjectOutputStream(aSocket.getOutputStream());
            ObjectInputStream inFromServer = new ObjectInputStream(aSocket.getInputStream());
            String nextMessage = "";
            while (!nextMessage.contains("winner")) {
                nextMessage = (String) inFromServer.readObject();
                System.out.println(nextMessage);
            }
        }
    }

    /*
     * @Before
     * public void setUp() throws Exception {
     * this.game = new BoomerangAustralia(new String[] { "1", "3" });
     * outContent = new ByteArrayOutputStream();
     * System.setOut(new PrintStream(outContent));
     * }
     */

    @Test
    public void req_1() throws Exception {

        ByteArrayOutputStream outContent = new ByteArrayOutputStream();
        System.setOut(new PrintStream(outContent));
        BoomerangAustralia game = new BoomerangAustralia(new String[] { "3", "2" });

        // Get first string, it is the error message we have some problems
        String capturedOutput = this.outContent.toString();
        System.out.println(capturedOutput);
        assertTrue(capturedOutput.contains("This game is for a total of 2-4 players/bots"));
    }
    /*
    @Test
    public void req_3() throws Exception {
        BoomerangAustralia game = new BoomerangAustralia(new String[] { "7", "2" });
        // Since the game is started we just check if hand is filled with 7 cards
        assertEquals(game.players.get(0).hand.size(), 7);
    }

    @Test
    public void req_2() throws Exception {
        BoomerangAustralia game = new BoomerangAustralia(new String[] { "7", "2" });
        assertEquals(game.arrayDeck.length, 28);

        for (Card card : game.arrayDeck) {
            if (card.name.equals("The Bungle Bungles")) {
                assertEquals("A", card.letter);
                assertEquals("Western Australia", card.region);
                assertEquals("Leaves", card.Collections);
                assertEquals("Indigenous Culture", card.Activities);
            }

            if (card.name.equals("The Pinnacles")) {
                assertEquals("B", card.letter);
                assertEquals("Western Australia", card.region);
                assertEquals("Kangaroos", card.Animals);
                assertEquals("Sightseeing", card.Activities);
            }

            if (card.name.equals("Margaret River")) {
                assertEquals("C", card.letter);
                assertEquals("Western Australia", card.region);
                assertEquals("Shells", card.Collections);
                assertEquals("Kangaroos", card.Animals);
            }

            if (card.name.equals("Kalbarri National Park")) {
                assertEquals("D", card.letter);
                assertEquals("Western Australia", card.region);
                assertEquals("Wildflowers", card.Collections);
                assertEquals("Bushwalking", card.Activities);
            }

            if (card.name.equals("Uluru")) {
                assertEquals("E", card.letter);
                assertEquals("Northern Territory", card.region);
                assertEquals("Emus", card.Animals);
                assertEquals("Indigenous Culture", card.Activities);
            }

            if (card.name.equals("Kakadu National Park")) {
                assertEquals("F", card.letter);
                assertEquals("Northern Territory", card.region);
                assertEquals("Wombats", card.Animals);
                assertEquals("Sightseeing", card.Activities);
            }

            if (card.name.equals("Nitmiluk National Park")) {
                assertEquals("G", card.letter);
                assertEquals("Northern Territory", card.region);
                assertEquals("Shells", card.Collections);
                assertEquals("Platypuses", card.Animals);
            }

            if (card.name.equals("King's Canyon")) {
                assertEquals("H", card.letter);
                assertEquals("Northern Territory", card.region);
                assertEquals("Koalas", card.Animals);
                assertEquals("Swimming", card.Activities);
            }

            if (card.name.equals("The Great Barrier Reef")) {
                assertEquals("I", card.letter);
                assertEquals("Queensland", card.region);
                assertEquals("Wildflowers", card.Collections);
                assertEquals("Sightseeing", card.Activities);
            }

            if (card.name.equals("The Whitsundays")) {
                assertEquals("J", card.letter);
                assertEquals("Queensland", card.region);
                assertEquals("Kangaroos", card.Animals);
                assertEquals("Indigenous Culture", card.Activities);
            }

            if (card.name.equals("Daintree Rainforest")) {
                assertEquals("K", card.letter);
                assertEquals("Queensland", card.region);
                assertEquals("Souvenirs", card.Collections);
                assertEquals("Bushwalking", card.Activities);
            }

            if (card.name.equals("Surfers Paradise")) {
                assertEquals("L", card.letter);
                assertEquals("Queensland", card.region);
                assertEquals("Wildflowers", card.Collections);
                assertEquals("Swimming", card.Activities);
            }

            if (card.name.equals("Barossa Valley")) {
                assertEquals("M", card.letter);
                assertEquals("South Australia", card.region);
                assertEquals("Koalas", card.Animals);
                assertEquals("Bushwalking", card.Activities);
            }

            if (card.name.equals("Lake Eyre")) {
                assertEquals("N", card.letter);
                assertEquals("South Australia", card.region);
                assertEquals("Emus", card.Animals);
                assertEquals("Swimming", card.Activities);
            }

            if (card.name.equals("Kangaroo Island")) {
                assertEquals("O", card.letter);
                assertEquals("South Australia", card.region);
                assertEquals("Kangaroos", card.Animals);
                assertEquals("Bushwalking", card.Activities);
            }

            if (card.name.equals("Mount Gambier")) {
                assertEquals("P", card.letter);
                assertEquals("South Australia", card.region);
                assertEquals("Wildflowers", card.Collections);
                assertEquals("Sightseeing", card.Activities);
            }

            if (card.name.equals("Blue Mountains")) {
                assertEquals("Q", card.letter);
                assertEquals("New South Whales", card.region);
                assertEquals("Indigenous Culture", card.Activities);
            }

            if (card.name.equals("Sydney Harbour")) {
                assertEquals("R", card.letter);
                assertEquals("New South Whales", card.region);
                assertEquals("Emus", card.Animals);
                assertEquals("Sightseeing", card.Activities);
            }

            if (card.name.equals("Bondi Beach")) {
                assertEquals("S", card.letter);
                assertEquals("New South Whales", card.region);
                assertEquals("Swimming", card.Activities);
            }

            if (card.name.equals("Hunter Valley")) {
                assertEquals("T", card.letter);
                assertEquals("New South Whales", card.region);
                assertEquals("Emus", card.Animals);
                assertEquals("Bushwalking", card.Activities);
            }

            if (card.name.equals("Melbourne")) {
                assertEquals("U", card.letter);
                assertEquals("Victoria", card.region);
                assertEquals("Wombats", card.Animals);
                assertEquals("Bushwalking", card.Activities);
            }

            if (card.name.equals("The MCG")) {
                assertEquals("V", card.letter);
                assertEquals("Victoria", card.region);
                assertEquals("Leaves", card.Collections);
                assertEquals("Indigenous Culture", card.Activities);
            }

            if (card.name.equals("Twelve Apostles")) {
                assertEquals("W", card.letter);
                assertEquals("Victoria", card.region);
                assertEquals("Shells", card.Collections);
                assertEquals("Swimming", card.Activities);
            }

            if (card.name.equals("Royal Exhibition Building")) {
                assertEquals("X", card.letter);
                assertEquals("Victoria", card.region);
                assertEquals("Leaves", card.Collections);
            }

            if (card.name.equals("Salamanca Markets")) {
                assertEquals("Y", card.letter);
                assertEquals("Tasmania", card.region);
                assertEquals("Leaves", card.Collections);
                assertEquals("Emus", card.Animals);
            }

            if (card.name.equals("Mount Wellington")) {
                assertEquals("Z", card.letter);
                assertEquals("Tasmania", card.region);
                assertEquals("Koalas", card.Animals);
                assertEquals("Sightseeing", card.Activities);
            }

            if (card.name.equals("Port Arthur")) {
                assertEquals("*", card.letter);
                assertEquals("Tasmania", card.region);
                assertEquals("Leaves", card.Collections);
                assertEquals("Indigenous Culture", card.Activities);
            }

            if (card.name.equals("Richmond")) {
                assertEquals("-", card.letter);
                assertEquals("Tasmania", card.region);
                assertEquals("Kangaroos", card.Animals);
                assertEquals("Swimming", card.Activities);
            }
        }
    }*/
}
