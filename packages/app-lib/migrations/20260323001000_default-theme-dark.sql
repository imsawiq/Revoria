UPDATE settings
SET theme = 'dark'
WHERE id = 0
  AND theme = 'oled'
  AND onboarded = FALSE;
