import { NativeModules } from 'react-native';
const { PlayerWalletModule } = NativeModules
interface PlayerWalletInterface {
   player_mine(): void;
}
export default PlayerWalletModule as PlayerWalletInterface;
