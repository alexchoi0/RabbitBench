declare global {
	namespace App {
		interface Locals {
			session: {
				user: {
					id: string;
					email: string;
					name: string | null;
					image: string | null;
				};
				token: string;
			} | null;
		}
	}
}

export {};
