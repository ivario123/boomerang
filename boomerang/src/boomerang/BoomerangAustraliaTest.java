package boomerang;

import static org.junit.Assert.*;
import org.junit.Before;
import org.junit.Test;
import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.util.ArrayList;
import java.util.Arrays;

public class BoomerangAustraliaTest {
    private BoomerangAustralia game;
    private ByteArrayOutputStream outContent;
    
    @Before
    public void setUp() {
        game = new BoomerangAustralia(new String[]{"4", "2"});
        outContent = new ByteArrayOutputStream();
        System.setOut(new PrintStream(outContent));
    }

    @Test
    public void testGameInitialization() {
        assertNotNull(game);
    }

    @Test
    public void testAddPlayerToDraft() {
        BoomerangAustralia.Player player = game.new Player(1, false, null, null, null);
        player.hand = new ArrayList<BoomerangAustralia.Card>(
            Arrays.asList(
                game.arrayDeck[0],
                game.arrayDeck[1],
                game.arrayDeck[2],
                game.arrayDeck[3],
                game.arrayDeck[4],
                game.arrayDeck[5],
                game.arrayDeck[6]
            )
        );
        BoomerangAustralia.Player sendToPlayer = game.new Player(2, false, null, null, null);
        player.addCardToDraft(sendToPlayer);
        assertEquals(6, player.draft.size());
        assertEquals(1, sendToPlayer.nextHand.size());
    }

    @Test
    public void testCheckRegionComplete() {
        BoomerangAustralia.Player player = game.new Player(1, false, null, null, null);
        player.draft = new ArrayList<BoomerangAustralia.Card>(
            Arrays.asList(
                game.arrayDeck[0],
                game.arrayDeck[1],
                game.arrayDeck[2],
                game.arrayDeck[3]
            )
        );
        boolean complete = player.checkRegionComplete("Western Australia");
        assertTrue(complete);
    }

    @Test
    public void testNumberThings() {
        BoomerangAustralia.Player player = game.new Player(1, false, null, null, null);
        player.draft = new ArrayList<BoomerangAustralia.Card>(
            Arrays.asList(
                game.arrayDeck[0],
                game.arrayDeck[1],
                game.arrayDeck[2],
                game.arrayDeck[3],
                game.arrayDeck[4],
                game.arrayDeck[5],
                game.arrayDeck[6]
            )
        );
        int count = player.numberThings("Leaves", "Collections");
        assertEquals(1, count);
    }

    @Test
    public void testRoundScore() {
        BoomerangAustralia.Player player = game.new Player(1, false, null, null, null);
        player.draft = new ArrayList<BoomerangAustralia.Card>(
            Arrays.asList(
                game.arrayDeck[0],
                game.arrayDeck[1],
                game.arrayDeck[2],
                game.arrayDeck[3],
                game.arrayDeck[4],
                game.arrayDeck[5],
                game.arrayDeck[6]
            )
        );
        int score = player.roundScore(1);
        assertTrue(score > 0);
    }

    @Test
    public void testBoomerangAustraliaMain() {
        // Test the main method. Since it's not directly testable, this test only checks for exceptions.
        String[] args = {"4", "2"};
        try {
            BoomerangAustralia.main(args);
        } catch (Exception e) {
            fail("An exception occurred: " + e.getMessage());
        }
    }
}
