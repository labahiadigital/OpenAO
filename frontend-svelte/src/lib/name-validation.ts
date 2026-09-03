const NAME_MIN_LENGTH = 3;
const NAME_MAX_LENGTH = 20;
const NAME_PATTERN = /^[a-zA-ZáéíóúÁÉÍÓÚñÑ\s]+$/;

export function validateCharacterName(
  name: string,
): { valid: true } | { valid: false; reason: string } {
  const trimmed = name.trim();

  if (trimmed.length < NAME_MIN_LENGTH) {
    return { valid: false, reason: `El nombre debe tener al menos ${NAME_MIN_LENGTH} caracteres.` };
  }

  if (trimmed.length > NAME_MAX_LENGTH) {
    return { valid: false, reason: `El nombre no puede tener más de ${NAME_MAX_LENGTH} caracteres.` };
  }

  if (!NAME_PATTERN.test(trimmed)) {
    return { valid: false, reason: "El nombre solo puede contener letras y espacios." };
  }

  return { valid: true };
}
