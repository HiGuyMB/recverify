//-----------------------------------------------------------------------------
// REC verification mod
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
package recverify {

function displayHelp() {
   Parent::displayHelp();
   error(
      "Rec Verify Mod options:\n"@
      "  -verify                Play a REC file and print information upon completion\n"
   );
}

function parseArgs()
{
   // Call the parent
   Parent::parseArgs();

   // Arguments, which override everything else.
   for (%i = 1; %i < $Game::argc ; %i++)
   {
      %arg = $Game::argv[%i];
      %nextArg = $Game::argv[%i+1];
      %hasNextArg = $Game::argc - %i > 1;
   
      switch$ (%arg)
      {
         case "-verify":
            $argUsed[%i]++;
            if(%hasNextArg) {
               $verifyDemo = true;
               $demoArg = %nextArg;
               $argUsed[%i+1]++;
               %i++;
               if (!isFile($demoArg)) {
                  error("Error: Verify argument does not exist");
                  quit();
               }
            }
            else
               error("Error: Missing Command Line argument. Usage: -verify <rec filename>");
      }
   }
}

function onStart()
{
   Parent::onStart();
   echo("\n--------- Initializing MOD: REC Verify ---------");

   if ($demoArg !$= "") {
      // Make sure client inits first
      schedule(10, 0, playDemo, $demoArg);
   }
}

function onExit()
{
   Parent::onExit();
}

function clientCmdGameEnd()
{
   if ($verifyDemo) {
      // Dump rec stats to stdout
      echo("DEMO VERIFY SUCCESS");
      echo("DEMO: " @ $demoArg);
      echo("MISSION: " @ $Server::MissionFile);
      echo("LEVEL NAME: " @ MissionInfo.name);
      echo("SCORE TIME: " @ ($Game::ScoreTime $= "" ? 0 : $Game::ScoreTime));
      echo("ELAPSED TIME: " @ ($Game::ElapsedTime $= "" ? 0 : $Game::ElapsedTime));
      echo("BONUS TIME: " @ ($Game::BonusTime $= "" ? 0 : $Game::BonusTime));
      echo("GEM COUNT: " @ (PlayGui.gemCount $= "" ? 0 : PlayGui.gemCount));
      echo("MAX GEMS: " @ (PlayGui.maxGems $= "" ? 0 : PlayGui.maxGems));
      quit();
   }
   Parent::clientCmdGameEnd();
}

function onDemoPlayDone(%forced)
{
   if ($verifyDemo) {
      // If we get here then we didn't reach success, so it's a failure
      echo("DEMO VERIFY FAILED");
      echo("DEMO: " @ $demoArg);
      echo("MISSION: " @ $Server::MissionFile);
      echo("LEVEL NAME: " @ MissionInfo.name);
      echo("SCORE TIME: " @ ($Game::ScoreTime $= "" ? 0 : $Game::ScoreTime));
      echo("ELAPSED TIME: " @ ($Game::ElapsedTime $= "" ? 0 : $Game::ElapsedTime));
      echo("BONUS TIME: " @ ($Game::BonusTime $= "" ? 0 : $Game::BonusTime));
      echo("GEM COUNT: " @ (PlayGui.gemCount $= "" ? 0 : PlayGui.gemCount));
      echo("MAX GEMS: " @ (PlayGui.maxGems $= "" ? 0 : PlayGui.maxGems));
      quit();
   }

   Parent::onDemoPlayDone(%forced);
}


}; // Client package
activatePackage(recverify);
